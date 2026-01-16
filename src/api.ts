import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
import { Account, Settings } from './store';

export interface NewAccount {
  email: string;
  email_password: string;
  client_id: string;
  refresh_token: string;
}

export interface AccountUpdate {
  id: number;
  email?: string;
  email_password?: string;
  client_id?: string;
  refresh_token?: string;
  kiro_password?: string;
  status?: 'not_registered' | 'in_progress' | 'registered' | 'error';
  error_reason?: string;
}

export interface ImportResult {
  success_count: number;
  error_count: number;
  errors: Array<{
    line_number: number;
    content: string;
    reason: string;
  }>;
}

export const api = {
  async getAccounts(statusFilter?: string): Promise<Account[]> {
    return invoke('get_accounts', { statusFilter });
  },

  async addAccount(account: NewAccount): Promise<number> {
    return invoke('add_account', { account });
  },

  async updateAccount(update: AccountUpdate): Promise<void> {
    return invoke('update_account', { update });
  },

  async deleteAccount(id: number): Promise<void> {
    return invoke('delete_account', { id });
  },

  async deleteAllAccounts(): Promise<void> {
    return invoke('delete_all_accounts');
  },

  async importAccounts(content: string): Promise<ImportResult> {
    return invoke('import_accounts', { content });
  },

  async getSettings(): Promise<Settings> {
    return invoke('get_settings');
  },

  async updateSettings(settings: Settings): Promise<void> {
    return invoke('update_settings', { settings });
  },

  async startRegistration(accountId: number): Promise<string> {
    return invoke('start_registration', { accountId });
  },

  async startBatchRegistration(): Promise<string> {
    return invoke('start_batch_registration');
  },

  async startBatchRegistrationByIds(accountIds: number[]): Promise<string> {
    return invoke('start_batch_registration_by_ids', { accountIds });
  },

  async exportAccounts(statusFilter?: string): Promise<void> {
    const content: string = await invoke('export_accounts', { statusFilter });

    if (!content) {
      throw new Error('没有可导出的数据');
    }

    const filePath = await save({
      filters: [{
        name: 'Text Files',
        extensions: ['txt']
      }],
      defaultPath: 'accounts.txt'
    });

    if (filePath) {
      await writeTextFile(filePath, content);
    }
  },

  async exportAccountsJson(statusFilter?: string): Promise<void> {
    const content: string = await invoke('export_accounts_json', { statusFilter });

    if (!content || content === '[]') {
      throw new Error('没有已注册的账号可导出');
    }

    const filePath = await save({
      filters: [{
        name: 'JSON Files',
        extensions: ['json']
      }],
      defaultPath: 'kiro_accounts.json'
    });

    if (filePath) {
      await writeTextFile(filePath, content);
    }
  },

  async selectFile(): Promise<string | null> {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Text Files',
        extensions: ['txt']
      }]
    });

    if (selected && typeof selected === 'string') {
      const content = await readTextFile(selected);
      return content;
    }

    return null;
  },

  async extractKiroToken(accountId: number): Promise<string> {
    return invoke('extract_kiro_token', { accountId });
  },

  async batchExtractKiroTokens(): Promise<string> {
    return invoke('batch_extract_kiro_tokens');
  },

  async exportSingleAccountJson(accountId: number): Promise<string> {
    return invoke('export_single_account_json', { accountId });
  },

  async getLatestVerificationCode(accountId: number): Promise<{ code: string; timestamp: number }> {
    return invoke('get_latest_verification_code', { accountId });
  },

  async updateEmailCredentials(content: string): Promise<string> {
    return invoke('update_email_credentials', { content });
  },

  async generateSsoAuthUrl(): Promise<string> {
    return invoke('generate_sso_auth_url');
  },

  async pollSsoToken(accountId: number, deviceCode: string, clientId: string, clientSecret: string, interval: number): Promise<string> {
    return invoke('poll_sso_token', { accountId, deviceCode, clientId, clientSecret, interval });
  }
};
