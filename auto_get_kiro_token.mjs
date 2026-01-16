/**
 * 自动获取 Kiro RefreshToken
 * 
 * 流程：
 * 1. 打开浏览器登录 AWS Builder ID
 * 2. 自动输入邮箱
 * 3. 通过 IMAP 获取验证码
 * 4. 自动输入验证码完成登录
 * 5. 提取 x-amz-sso_authn token
 * 6. 执行 SSO 设备授权流程获取 refreshToken
 */

import puppeteer from 'puppeteer';
import Imap from 'imap';
import { simpleParser } from 'mailparser';
import fs from 'fs';

// ========== 配置 ==========
const EMAIL = 'itfbvn94310@outlook.com';
const OUTLOOK_CLIENT_ID = '9e5f94bc-e8a4-4e73-b8be-63364c29d753';
const OUTLOOK_REFRESH_TOKEN = 'M.C540_BAY.0.U.-CqoxQgF1KVuaIhVz!SYPiFtJYkwxIVdfCq8R5Gilaaf7koG4eX52EGQ5ISZ*EPoFFCwvR0O3CjHa6QRj46LTdnid52ZRZTQzsHD5cdfnaojLW22l0udKJRM9uOHYsEsYYO2nzsx5XRlIoYQj804pPMGY9mYhy3fSb3HDUyIAIFfZmga2z9l9Gof3NhqrVeDhpOK7DzAJC9Gf52tMIY28T92OgHUmVAZbg4p2vuoNjWhh8dp2EuGkLD5PT42q1y1aFvgJzyCLoF5Ef56Va9zdw*Yhz9rzBRcvgNdVlNr5sh3SHLNzWW4qxB*cSNkCC*CGb6UKJMKc4vLp0GAV8p6avK88sElppLgpeWWR1YECjSaGOidMWqIRsVE9Yzd9XLSbYSRBaivnitqNiOZuJMpORL72pvxi1ybIgBpvHVSgeMG*';
// ========== 配置结束 ==========

const OIDC_BASE = 'https://oidc.us-east-1.amazonaws.com';
const PORTAL_BASE = 'https://portal.sso.us-east-1.amazonaws.com';
const START_URL = 'https://view.awsapps.com/start';
const SCOPES = ['codewhisperer:analysis', 'codewhisperer:completions', 'codewhisperer:conversations', 'codewhisperer:taskassist', 'codewhisperer:transformations'];

// 获取 Outlook Access Token
async function getOutlookAccessToken() {
  console.log('🔑 获取 Outlook Access Token...');
  const response = await fetch('https://login.microsoftonline.com/common/oauth2/v2.0/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({
      client_id: OUTLOOK_CLIENT_ID,
      refresh_token: OUTLOOK_REFRESH_TOKEN,
      grant_type: 'refresh_token',
      scope: 'https://outlook.office.com/IMAP.AccessAsUser.All offline_access'
    })
  });
  const data = await response.json();
  if (data.error) throw new Error(data.error_description);
  return data.access_token;
}

// 通过 IMAP 获取最新验证码
async function getVerificationCode(accessToken, afterTime) {
  console.log('📬 检查邮箱验证码...');
  return new Promise((resolve, reject) => {
    const xoauth2Token = Buffer.from(`user=${EMAIL}\x01auth=Bearer ${accessToken}\x01\x01`).toString('base64');
    
    const imap = new Imap({
      user: EMAIL,
      xoauth2: xoauth2Token,
      host: 'outlook.office365.com',
      port: 993,
      tls: true,
      tlsOptions: { rejectUnauthorized: false }
    });

    imap.once('ready', () => {
      imap.openBox('INBOX', false, (err) => {
        if (err) { imap.end(); return reject(err); }

        imap.search(['ALL'], (err, results) => {
          if (err || !results?.length) { imap.end(); return reject(new Error('No emails')); }
          
          const recent = results.slice(-10);
          const emails = [];
          const fetch = imap.fetch(recent, { bodies: '' });

          fetch.on('message', (msg) => {
            msg.on('body', (stream) => {
              simpleParser(stream, (err, parsed) => {
                if (!err) {
                  emails.push({ subject: parsed.subject, from: parsed.from?.text, date: parsed.date, text: parsed.text });
                }
              });
            });
          });

          fetch.once('end', () => {
            imap.end();
            // 找最新的 AWS 验证码邮件
            emails.sort((a, b) => new Date(b.date) - new Date(a.date));
            for (const email of emails) {
              if (email.from?.includes('awsapps.com') && new Date(email.date) > afterTime) {
                const match = email.text?.match(/\b(\d{6})\b/);
                if (match) {
                  console.log('✅ 找到验证码:', match[1], '时间:', email.date);
                  return resolve(match[1]);
                }
              }
            }
            resolve(null);
          });
        });
      });
    });

    imap.once('error', reject);
    imap.connect();
  });
}

// SSO 设备授权流程
async function ssoDeviceAuth(bearerToken) {
  let clientId, clientSecret, deviceCode, userCode, deviceSessionToken, interval = 1;

  // Step 1: 注册 OIDC 客户端
  console.log('[SSO] Step 1: 注册 OIDC 客户端...');
  const regRes = await fetch(`${OIDC_BASE}/client/register`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ clientName: 'Kiro Account Manager', clientType: 'public', scopes: SCOPES, grantTypes: ['urn:ietf:params:oauth:grant-type:device_code', 'refresh_token'], issuerUrl: START_URL })
  });
  if (!regRes.ok) throw new Error(`注册失败: ${regRes.status}`);
  const regData = await regRes.json();
  clientId = regData.clientId;
  clientSecret = regData.clientSecret;
  console.log('[SSO] 客户端注册成功');

  // Step 2: 发起设备授权
  console.log('[SSO] Step 2: 发起设备授权...');
  const devRes = await fetch(`${OIDC_BASE}/device_authorization`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ clientId, clientSecret, startUrl: START_URL })
  });
  if (!devRes.ok) throw new Error(`设备授权失败: ${devRes.status}`);
  const devData = await devRes.json();
  deviceCode = devData.deviceCode;
  userCode = devData.userCode;
  interval = devData.interval || 1;
  console.log('[SSO] 设备码获取成功, user_code:', userCode);

  // Step 3: 验证 Bearer Token
  console.log('[SSO] Step 3: 验证 Bearer Token...');
  const whoRes = await fetch(`${PORTAL_BASE}/token/whoAmI`, {
    method: 'GET',
    headers: { 'Authorization': `Bearer ${bearerToken}`, 'Accept': 'application/json' }
  });
  if (!whoRes.ok) throw new Error(`whoAmI 失败: ${whoRes.status}`);
  console.log('[SSO] Bearer Token 验证成功');

  // Step 4: 获取设备会话令牌
  console.log('[SSO] Step 4: 获取设备会话令牌...');
  const sessRes = await fetch(`${PORTAL_BASE}/session/device`, {
    method: 'POST',
    headers: { 'Authorization': `Bearer ${bearerToken}`, 'Content-Type': 'application/json' },
    body: JSON.stringify({})
  });
  if (!sessRes.ok) throw new Error(`设备会话失败: ${sessRes.status}`);
  deviceSessionToken = (await sessRes.json()).token;
  console.log('[SSO] 设备会话令牌获取成功');

  // Step 5: 接受用户代码
  console.log('[SSO] Step 5: 接受用户代码...');
  const acceptRes = await fetch(`${OIDC_BASE}/device_authorization/accept_user_code`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'Referer': 'https://view.awsapps.com/' },
    body: JSON.stringify({ userCode, userSessionId: deviceSessionToken })
  });
  if (!acceptRes.ok) throw new Error(`接受用户代码失败: ${acceptRes.status}`);
  const deviceContext = (await acceptRes.json()).deviceContext;
  console.log('[SSO] 用户代码接受成功');

  // Step 6: 批准授权
  if (deviceContext?.deviceContextId) {
    console.log('[SSO] Step 6: 批准授权...');
    const approveRes = await fetch(`${OIDC_BASE}/device_authorization/associate_token`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Referer': 'https://view.awsapps.com/' },
      body: JSON.stringify({ deviceContext: { deviceContextId: deviceContext.deviceContextId, clientId: deviceContext.clientId || clientId, clientType: deviceContext.clientType || 'public' }, userSessionId: deviceSessionToken })
    });
    if (!approveRes.ok) throw new Error(`批准失败: ${approveRes.status}`);
    console.log('[SSO] 授权批准成功');
  }

  // Step 7: 轮询获取 Token
  console.log('[SSO] Step 7: 轮询获取 Token...');
  const startTime = Date.now();
  while (Date.now() - startTime < 120000) {
    await new Promise(r => setTimeout(r, interval * 1000));
    const tokenRes = await fetch(`${OIDC_BASE}/token`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ clientId, clientSecret, grantType: 'urn:ietf:params:oauth:grant-type:device_code', deviceCode })
    });
    if (tokenRes.ok) {
      const tokenData = await tokenRes.json();
      console.log('[SSO] Token 获取成功!');
      return { success: true, accessToken: tokenData.accessToken, refreshToken: tokenData.refreshToken, clientId, clientSecret, expiresIn: tokenData.expiresIn };
    }
    if (tokenRes.status === 400) {
      const errData = await tokenRes.json();
      if (errData.error === 'authorization_pending') { process.stdout.write('.'); continue; }
      if (errData.error === 'slow_down') { interval += 5; continue; }
      throw new Error(`Token 获取失败: ${errData.error}`);
    }
  }
  throw new Error('授权超时');
}

async function main() {
  console.log('🚀 自动获取 Kiro RefreshToken\n');
  console.log('账号:', EMAIL, '\n');

  const outlookToken = await getOutlookAccessToken();
  console.log('✅ Outlook Token 获取成功\n');

  const browser = await puppeteer.launch({
    headless: false,
    defaultViewport: { width: 1280, height: 900 },
    args: ['--no-sandbox']
  });

  const page = await browser.newPage();
  
  // 导航到 Kiro 登录
  console.log('🌐 打开 Kiro 登录页...');
  await page.goto('https://kiro.dev/signin', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));

  // 点击 AWS Builder ID 登录
  console.log('👆 点击 AWS Builder ID 登录...');
  try {
    await page.click('button:has-text("AWS Builder ID")');
  } catch {
    // 尝试其他选择器
    const buttons = await page.$$('button');
    for (const btn of buttons) {
      const text = await btn.evaluate(el => el.textContent);
      if (text?.includes('Builder') || text?.includes('AWS')) {
        await btn.click();
        break;
      }
    }
  }
  await new Promise(r => setTimeout(r, 3000));

  // 等待跳转到 AWS 登录页
  console.log('⏳ 等待 AWS 登录页...');
  await page.waitForNavigation({ waitUntil: 'networkidle2', timeout: 30000 }).catch(() => {});
  
  // 输入邮箱
  console.log('📧 输入邮箱:', EMAIL);
  const emailInput = await page.$('input[type="email"], input[name="email"], input[placeholder*="email"]');
  if (emailInput) {
    await emailInput.type(EMAIL, { delay: 50 });
    await new Promise(r => setTimeout(r, 500));
    
    // 点击下一步
    const nextBtn = await page.$('button[type="submit"], button:has-text("Next"), button:has-text("继续")');
    if (nextBtn) await nextBtn.click();
  }

  // 记录发送验证码的时间
  const codeRequestTime = new Date();
  console.log('⏰ 验证码请求时间:', codeRequestTime.toISOString());

  // 等待验证码页面
  await new Promise(r => setTimeout(r, 5000));

  // 轮询获取验证码
  console.log('\n📬 等待验证码邮件...');
  let code = null;
  for (let i = 0; i < 30; i++) {
    await new Promise(r => setTimeout(r, 3000));
    code = await getVerificationCode(outlookToken, codeRequestTime).catch(() => null);
    if (code) break;
    process.stdout.write('.');
  }

  if (!code) {
    console.log('\n❌ 未能获取验证码，请手动输入');
    // 保持浏览器打开让用户手动操作
    await new Promise(() => {});
  }

  // 输入验证码
  console.log('\n🔢 输入验证码:', code);
  const codeInput = await page.$('input[type="text"], input[name="code"], input[placeholder*="code"]');
  if (codeInput) {
    await codeInput.type(code, { delay: 100 });
    await new Promise(r => setTimeout(r, 500));
    
    // 点击继续
    const submitBtn = await page.$('button[type="submit"], button:has-text("Continue"), button:has-text("继续")');
    if (submitBtn) await submitBtn.click();
  }

  // 等待登录完成
  console.log('⏳ 等待登录完成...');
  await new Promise(r => setTimeout(r, 10000));

  // 检查是否登录成功，提取 x-amz-sso_authn
  const cookies = await page.cookies();
  const ssoAuthCookie = cookies.find(c => c.name === 'x-amz-sso_authn');

  if (ssoAuthCookie) {
    console.log('\n✅ 登录成功! 提取到 x-amz-sso_authn token');
    console.log('Token 长度:', ssoAuthCookie.value.length);

    // 执行 SSO 设备授权
    console.log('\n🔄 开始 SSO 设备授权流程...\n');
    const result = await ssoDeviceAuth(ssoAuthCookie.value);

    if (result.success) {
      console.log('\n' + '='.repeat(60));
      console.log('✅ 成功获取 Kiro Token!');
      console.log('='.repeat(60));
      console.log('AccessToken:', result.accessToken?.substring(0, 50) + '...');
      console.log('RefreshToken:', result.refreshToken?.substring(0, 50) + '...');
      console.log('ClientId:', result.clientId?.substring(0, 30) + '...');
      console.log('='.repeat(60));

      if (result.refreshToken?.startsWith('aor')) {
        console.log('\n✅ RefreshToken 格式正确 (aor 开头)');
      }

      // 保存到文件
      const tokenData = {
        email: EMAIL,
        accessToken: result.accessToken,
        refreshToken: result.refreshToken,
        clientId: result.clientId,
        clientSecret: result.clientSecret,
        region: 'us-east-1',
        provider: 'BuilderId',
        authMethod: 'IdC',
        expiresIn: result.expiresIn,
        createdAt: new Date().toISOString()
      };
      fs.writeFileSync('kiro_token.json', JSON.stringify(tokenData, null, 2));
      console.log('\n📁 Token 已保存到 kiro_token.json');
    }
  } else {
    console.log('\n❌ 未找到 x-amz-sso_authn cookie');
    console.log('当前 URL:', page.url());
    console.log('请手动完成登录...');
  }

  await browser.close();
}

main().catch(console.error);
