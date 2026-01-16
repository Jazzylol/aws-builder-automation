import { useState, useMemo } from 'react';
import { Trash2, Edit, Eye, Play, Loader2, Key, PlayCircle, Square, CheckSquare, Copy, RefreshCw } from 'lucide-react';
import { Account } from '../store';
import { api } from '../api';
import { showConfirm, showSuccess, showError } from '../utils/dialog';
import './AccountsTable.css';

interface AccountsTableProps {
  accounts: Account[];
  onRefresh: () => void;
}

export function AccountsTable({ accounts, onRefresh }: AccountsTableProps) {
  const [search, setSearch] = useState('');
  const [sortField, setSortField] = useState<keyof Account>('created_at');
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('desc');
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedAccount, setSelectedAccount] = useState<Account | null>(null);
  const [isDetailModalOpen, setIsDetailModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [processingId, setProcessingId] = useState<number | null>(null);
  const [extractingId, setExtractingId] = useState<number | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [isBatchProcessing, setIsBatchProcessing] = useState(false);
  
  // 验证码相关状态
  const [verificationCodes, setVerificationCodes] = useState<Map<number, { code: string; timestamp: number }>>(new Map());
  const [loadingCodes, setLoadingCodes] = useState<Set<number>>(new Set());
  
  // SSO 授权链接相关状态
  const [ssoAuthUrls, setSsoAuthUrls] = useState<Map<number, { url: string; deviceCode: string; clientId: string; clientSecret: string; interval: number }>>(new Map());
  const [loadingSsoUrls, setLoadingSsoUrls] = useState<Set<number>>(new Set());
  const [pollingSsoTokens, setPollingSsoTokens] = useState<Set<number>>(new Set());

  const itemsPerPage = 20;

  const filteredAccounts = useMemo(() => {
    return accounts.filter(account =>
      account.email.toLowerCase().includes(search.toLowerCase()) ||
      account.status.toLowerCase().includes(search.toLowerCase())
    );
  }, [accounts, search]);

  const sortedAccounts = useMemo(() => {
    return [...filteredAccounts].sort((a, b) => {
      const aValue = a[sortField];
      const bValue = b[sortField];

      if (aValue === null || aValue === undefined) return 1;
      if (bValue === null || bValue === undefined) return -1;

      if (typeof aValue === 'string' && typeof bValue === 'string') {
        return sortDirection === 'asc'
          ? aValue.localeCompare(bValue)
          : bValue.localeCompare(aValue);
      }

      return sortDirection === 'asc'
        ? (aValue > bValue ? 1 : -1)
        : (bValue > aValue ? 1 : -1);
    });
  }, [filteredAccounts, sortField, sortDirection]);

  const paginatedAccounts = useMemo(() => {
    const start = (currentPage - 1) * itemsPerPage;
    const end = start + itemsPerPage;
    return sortedAccounts.slice(start, end);
  }, [sortedAccounts, currentPage]);

  const totalPages = Math.ceil(sortedAccounts.length / itemsPerPage);

  // 可选中的账号（未注册、异常、进行中）
  const selectableAccounts = useMemo(() => {
    return sortedAccounts.filter(a => 
      a.status === 'not_registered' || a.status === 'error' || a.status === 'in_progress'
    );
  }, [sortedAccounts]);

  // 当前页可选中的账号
  const selectableOnPage = useMemo(() => {
    return paginatedAccounts.filter(a => 
      a.status === 'not_registered' || a.status === 'error' || a.status === 'in_progress'
    );
  }, [paginatedAccounts]);

  // 是否全选当前页
  const isAllSelectedOnPage = useMemo(() => {
    return selectableOnPage.length > 0 && selectableOnPage.every(a => selectedIds.has(a.id));
  }, [selectableOnPage, selectedIds]);

  // 判断账号是否可选
  const isSelectable = () => {
    // 所有状态的账号都可以选择
    return true;
  };

  // 全选/取消全选当前页
  const toggleSelectAllOnPage = () => {
    const newSet = new Set(selectedIds);
    if (isAllSelectedOnPage) {
      selectableOnPage.forEach(a => newSet.delete(a.id));
    } else {
      selectableOnPage.forEach(a => newSet.add(a.id));
    }
    setSelectedIds(newSet);
  };

  // 全选所有可选账号
  const selectAll = () => {
    const newSet = new Set<number>();
    selectableAccounts.forEach(a => newSet.add(a.id));
    setSelectedIds(newSet);
  };

  // 清空选择
  const clearSelection = () => {
    setSelectedIds(new Set());
  };

  // 批量注册选中的账号
  const handleBatchRegisterSelected = async () => {
    if (selectedIds.size === 0) {
      await showError('请先选择要注册的账号');
      return;
    }

    const confirmed = await showConfirm(
      `确定要对选中的 ${selectedIds.size} 个账号进行批量注册吗？`,
      '批量注册确认'
    );

    if (confirmed) {
      setIsBatchProcessing(true);
      try {
        const result = await api.startBatchRegistrationByIds(Array.from(selectedIds));
        await showSuccess(result);
        setSelectedIds(new Set());
        onRefresh();
      } catch (error) {
        await showError('批量注册失败: ' + error);
        onRefresh();
      } finally {
        setIsBatchProcessing(false);
      }
    }
  };

  // 批量提取选中账号的 Token
  const handleBatchExtractTokenSelected = async () => {
    if (selectedIds.size === 0) {
      await showError('请先选择要提取 Token 的账号');
      return;
    }

    // 检查选中的账号是否都是已注册状态
    const selectedAccounts = accounts.filter(a => selectedIds.has(a.id));
    const registeredAccounts = selectedAccounts.filter(a => a.status === 'registered');
    
    if (registeredAccounts.length === 0) {
      await showError('选中的账号中没有已注册的账号');
      return;
    }

    const confirmed = await showConfirm(
      `确定要对选中的 ${registeredAccounts.length} 个已注册账号提取 Token 吗？\n\n注意：每个账号都需要你手动完成 SSO 授权（点击确认并继续、允许访问按钮）`,
      '批量提取 Token 确认'
    );

    if (confirmed) {
      setIsBatchProcessing(true);
      let successCount = 0;
      let errorCount = 0;
      
      for (const account of registeredAccounts) {
        try {
          setExtractingId(account.id);
          await api.extractKiroToken(account.id);
          successCount++;
          onRefresh();
        } catch (error) {
          console.error(`提取 Token 失败 (${account.email}):`, error);
          errorCount++;
        }
      }
      
      setExtractingId(null);
      setIsBatchProcessing(false);
      setSelectedIds(new Set());
      
      await showSuccess(`批量提取完成！成功: ${successCount}, 失败: ${errorCount}`);
      onRefresh();
    }
  };

  const handleSort = (field: keyof Account) => {
    if (sortField === field) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDirection('asc');
    }
  };

  const handleDelete = async (id: number) => {
    const confirmed = await showConfirm('确定要删除这条记录吗?', '确认删除');
    if (confirmed) {
      try {
        await api.deleteAccount(id);
        onRefresh();
      } catch (error) {
        await showError('删除失败: ' + error);
      }
    }
  };

  const handleStartRegistration = async (id: number) => {
    if (processingId || extractingId) {
      return;
    }

    setProcessingId(id);
    try {
      await api.startRegistration(id);
      onRefresh();
    } catch (error) {
      onRefresh();
    } finally {
      setProcessingId(null);
    }
  };

  const handleExtractToken = async (id: number) => {
    if (processingId || extractingId) {
      return;
    }

    setExtractingId(id);
    try {
      const result = await api.extractKiroToken(id);
      await showSuccess(result);
      onRefresh();
    } catch (error) {
      await showError('获取Token失败: ' + error);
      onRefresh();
    } finally {
      setExtractingId(null);
    }
  };

  // 获取验证码
  const fetchVerificationCode = async (accountId: number) => {
    setLoadingCodes(prev => new Set(prev).add(accountId));
    try {
      const result = await api.getLatestVerificationCode(accountId);
      setVerificationCodes(prev => new Map(prev).set(accountId, result));
    } catch (error) {
      console.error(`获取验证码失败 (账号 ${accountId}):`, error);
      // 显示错误提示
      const errorMsg = String(error);
      if (errorMsg.includes('invalid_grant') || errorMsg.includes('Token request failed')) {
        await showError('该账号的邮箱 Token 已失效，请重新导入账号信息');
      } else {
        await showError(`获取验证码失败: ${error}`);
      }
    } finally {
      setLoadingCodes(prev => {
        const newSet = new Set(prev);
        newSet.delete(accountId);
        return newSet;
      });
    }
  };

  // 复制到剪贴板
  const copyToClipboard = async (text: string, label: string) => {
    try {
      await navigator.clipboard.writeText(text);
      await showSuccess(`${label}已复制到剪贴板`);
    } catch (error) {
      await showError(`复制失败: ${error}`);
    }
  };

  // 格式化相对时间
  const formatRelativeTime = (timestamp: number): string => {
    const now = Math.floor(Date.now() / 1000);
    const diff = now - timestamp;

    if (diff < 60) return '刚刚';
    if (diff < 3600) return `${Math.floor(diff / 60)}分钟前`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}小时前`;
    return `${Math.floor(diff / 86400)}天前`;
  };

  // 生成 SSO 授权链接
  const generateSsoAuthUrl = async (accountId: number) => {
    setLoadingSsoUrls(prev => new Set(prev).add(accountId));
    try {
      const result = await api.generateSsoAuthUrl();
      // 解析结果: url|||device_code|||client_id|||client_secret|||interval
      const parts = result.split('|||');
      if (parts.length === 5) {
        setSsoAuthUrls(prev => new Map(prev).set(accountId, {
          url: parts[0],
          deviceCode: parts[1],
          clientId: parts[2],
          clientSecret: parts[3],
          interval: parseInt(parts[4])
        }));
        await showSuccess('SSO 授权链接已生成');
      } else {
        throw new Error('返回数据格式错误');
      }
    } catch (error) {
      await showError(`生成授权链接失败: ${error}`);
    } finally {
      setLoadingSsoUrls(prev => {
        const newSet = new Set(prev);
        newSet.delete(accountId);
        return newSet;
      });
    }
  };

  // 轮询获取 SSO Token
  const pollSsoToken = async (accountId: number) => {
    const authData = ssoAuthUrls.get(accountId);
    if (!authData) {
      await showError('请先生成授权链接');
      return;
    }

    setPollingSsoTokens(prev => new Set(prev).add(accountId));
    try {
      const result = await api.pollSsoToken(
        accountId,
        authData.deviceCode,
        authData.clientId,
        authData.clientSecret,
        authData.interval
      );
      await showSuccess(result);
      onRefresh();
      // 清除授权链接数据
      setSsoAuthUrls(prev => {
        const newMap = new Map(prev);
        newMap.delete(accountId);
        return newMap;
      });
    } catch (error) {
      await showError(`获取 Token 失败: ${error}`);
    } finally {
      setPollingSsoTokens(prev => {
        const newSet = new Set(prev);
        newSet.delete(accountId);
        return newSet;
      });
    }
  };

  // 监听选中状态变化，自动获取验证码
  const handleToggleSelect = (id: number) => {
    const newSet = new Set(selectedIds);
    const wasSelected = newSet.has(id);
    
    if (wasSelected) {
      newSet.delete(id);
    } else {
      newSet.add(id);
      // 勾选时自动获取验证码
      fetchVerificationCode(id);
    }
    
    setSelectedIds(newSet);
  };

  const handleExportSingleJson = async (account: Account) => {
    try {
      const result = await api.exportSingleAccountJson(account.id);
      await showSuccess(result);
    } catch (error) {
      await showError('导出失败: ' + error);
    }
  };

  const getStatusText = (status: string) => {
    const statusMap: Record<string, string> = {
      not_registered: '未注册',
      in_progress: '进行中',
      registered: '已注册',
      error: '异常',
    };
    return statusMap[status] || status;
  };

  const getStatusClass = (status: string) => {
    return `status-badge status-${status.replace('_', '-')}`;
  };

  return (
    <div className="accounts-table-container">
      <div className="table-header">
        <input
          type="text"
          placeholder="搜索邮箱或状态..."
          value={search}
          onChange={(e) => {
            setSearch(e.target.value);
            setCurrentPage(1);
          }}
          className="search-input"
        />
        <div className="table-stats">
          共 {sortedAccounts.length} 条记录
          {selectedIds.size > 0 && (
            <span className="selected-count">，已选 {selectedIds.size} 条</span>
          )}
        </div>
      </div>

      {/* 批量操作栏 */}
      {selectableAccounts.length > 0 && (
        <div className="batch-actions-bar">
          <div className="batch-select-buttons">
            <button
              className="batch-select-button"
              onClick={toggleSelectAllOnPage}
              disabled={isBatchProcessing || selectableOnPage.length === 0}
            >
              {isAllSelectedOnPage ? <CheckSquare size={16} /> : <Square size={16} />}
              {isAllSelectedOnPage ? '取消本页' : '选择本页'}
            </button>
            <button
              className="batch-select-button"
              onClick={selectAll}
              disabled={isBatchProcessing}
            >
              全选所有 ({selectableAccounts.length})
            </button>
            {selectedIds.size > 0 && (
              <button
                className="batch-select-button batch-clear-button"
                onClick={clearSelection}
                disabled={isBatchProcessing}
              >
                清空选择
              </button>
            )}
          </div>
          {selectedIds.size > 0 && (
            <button
              className="batch-register-button"
              onClick={handleBatchRegisterSelected}
              disabled={isBatchProcessing || processingId !== null || extractingId !== null}
            >
              {isBatchProcessing ? (
                <>
                  <Loader2 size={16} className="spin" />
                  处理中...
                </>
              ) : (
                <>
                  <PlayCircle size={16} />
                  批量注册 ({selectedIds.size})
                </>
              )}
            </button>
          )}
          {selectedIds.size > 0 && (
            <button
              className="batch-extract-button"
              onClick={handleBatchExtractTokenSelected}
              disabled={isBatchProcessing || processingId !== null || extractingId !== null}
            >
              <Key size={16} />
              批量提取Token ({selectedIds.size})
            </button>
          )}
        </div>
      )}

      <div className="table-wrapper">
        <table className="accounts-table">
          <thead>
            <tr>
              <th className="checkbox-column">
                <button
                  className="checkbox-button"
                  onClick={toggleSelectAllOnPage}
                  disabled={selectableOnPage.length === 0}
                  title={isAllSelectedOnPage ? '取消全选' : '全选本页'}
                >
                  {isAllSelectedOnPage ? <CheckSquare size={18} /> : <Square size={18} />}
                </button>
              </th>
              <th onClick={() => handleSort('id')}>
                序号 {sortField === 'id' && (sortDirection === 'asc' ? '↑' : '↓')}
              </th>
              <th onClick={() => handleSort('email')}>
                注册邮箱 {sortField === 'email' && (sortDirection === 'asc' ? '↑' : '↓')}
              </th>
              <th>邮箱密码</th>
              <th onClick={() => handleSort('status')}>
                状态 {sortField === 'status' && (sortDirection === 'asc' ? '↑' : '↓')}
              </th>
              <th>Kiro 密码</th>
              <th>验证码</th>
              <th>SSO 授权</th>
              <th>异常原因</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            {paginatedAccounts.map((account, index) => (
              <tr key={account.id} className={selectedIds.has(account.id) ? 'row-selected' : ''}>
                <td className="checkbox-column">
                  {isSelectable() ? (
                    <button
                      className="checkbox-button"
                      onClick={() => handleToggleSelect(account.id)}
                      disabled={isBatchProcessing}
                    >
                      {selectedIds.has(account.id) ? (
                        <CheckSquare size={18} className="checkbox-checked" />
                      ) : (
                        <Square size={18} />
                      )}
                    </button>
                  ) : (
                    <span className="checkbox-disabled">-</span>
                  )}
                </td>
                <td>{(currentPage - 1) * itemsPerPage + index + 1}</td>
                <td className="email-cell">
                  <div className="email-with-copy">
                    <span>{account.email}</span>
                    <button
                      className="icon-button copy-button"
                      onClick={() => copyToClipboard(account.email, '邮箱')}
                      title="复制邮箱"
                    >
                      <Copy size={14} />
                    </button>
                  </div>
                </td>
                <td>
                  <span className="password-hidden">••••••••</span>
                </td>
                <td>
                  <span className={getStatusClass(account.status)}>
                    {getStatusText(account.status)}
                  </span>
                </td>
                <td className="password-cell">
                  {account.kiro_password ? (
                    <div className="password-with-copy">
                      <span className="password-hidden">••••••••</span>
                      <button
                        className="icon-button copy-button"
                        onClick={() => copyToClipboard(account.kiro_password!, 'Kiro 密码')}
                        title="复制 Kiro 密码"
                      >
                        <Copy size={14} />
                      </button>
                    </div>
                  ) : (
                    <span className="text-muted">-</span>
                  )}
                </td>
                <td className="verification-code-cell">
                  {loadingCodes.has(account.id) ? (
                    <div className="code-loading">
                      <Loader2 size={14} className="spin" />
                      <span>获取中...</span>
                    </div>
                  ) : verificationCodes.has(account.id) ? (
                    <div className="code-with-copy">
                      <div className="code-info">
                        <span className="code-text">{verificationCodes.get(account.id)!.code}</span>
                        <span className="code-time">{formatRelativeTime(verificationCodes.get(account.id)!.timestamp)}</span>
                      </div>
                      <div className="code-actions">
                        <button
                          className="icon-button copy-button"
                          onClick={() => copyToClipboard(verificationCodes.get(account.id)!.code, '验证码')}
                          title="复制验证码"
                        >
                          <Copy size={14} />
                        </button>
                        <button
                          className="icon-button refresh-button"
                          onClick={() => fetchVerificationCode(account.id)}
                          title="刷新验证码"
                        >
                          <RefreshCw size={14} />
                        </button>
                      </div>
                    </div>
                  ) : (
                    <span className="text-muted">-</span>
                  )}
                </td>
                <td className="sso-auth-cell">
                  {loadingSsoUrls.has(account.id) ? (
                    <div className="sso-loading">
                      <Loader2 size={14} className="spin" />
                      <span>生成中...</span>
                    </div>
                  ) : ssoAuthUrls.has(account.id) ? (
                    <div className="sso-auth-actions">
                      <button
                        className="icon-button copy-button"
                        onClick={() => copyToClipboard(ssoAuthUrls.get(account.id)!.url, 'SSO 授权链接')}
                        title="复制授权链接"
                      >
                        <Copy size={14} />
                      </button>
                      <button
                        className="icon-button poll-button"
                        onClick={() => pollSsoToken(account.id)}
                        disabled={pollingSsoTokens.has(account.id)}
                        title="获取 Token"
                      >
                        {pollingSsoTokens.has(account.id) ? (
                          <Loader2 size={14} className="spin" />
                        ) : (
                          <Key size={14} />
                        )}
                      </button>
                    </div>
                  ) : (
                    <button
                      className="generate-sso-button"
                      onClick={() => generateSsoAuthUrl(account.id)}
                      title="生成 SSO 授权链接"
                    >
                      生成链接
                    </button>
                  )}
                </td>
                <td className="error-cell">
                  {account.error_reason && (
                    <span className="error-text" title={account.error_reason}>
                      {account.error_reason.substring(0, 50)}
                      {account.error_reason.length > 50 && '...'}
                    </span>
                  )}
                </td>
                <td>
                  <div className="action-buttons">
                    <button
                      className="action-button"
                      onClick={() => {
                        setSelectedAccount(account);
                        setIsDetailModalOpen(true);
                      }}
                      title="查看详情"
                    >
                      <Eye size={16} />
                    </button>
                    <button
                      className="action-button"
                      onClick={() => {
                        setSelectedAccount(account);
                        setIsEditModalOpen(true);
                      }}
                      title="编辑"
                    >
                      <Edit size={16} />
                    </button>
                    {(account.status === 'not_registered' || account.status === 'error' || account.status === 'in_progress') && (
                      <button
                        className="action-button action-button-primary"
                        onClick={() => handleStartRegistration(account.id)}
                        title={account.status === 'error' ? '重试注册' : '开始注册'}
                        disabled={processingId === account.id || processingId !== null || extractingId !== null || isBatchProcessing}
                      >
                        {processingId === account.id ? (
                          <Loader2 size={16} className="spin" />
                        ) : (
                          <Play size={16} />
                        )}
                      </button>
                    )}
                    {account.status === 'registered' && account.kiro_password && (
                      <button
                        className="action-button action-button-warning"
                        onClick={() => handleExtractToken(account.id)}
                        title="提取 Kiro Token"
                        disabled={extractingId === account.id || extractingId !== null || processingId !== null || isBatchProcessing}
                      >
                        {extractingId === account.id ? (
                          <Loader2 size={16} className="spin" />
                        ) : (
                          <Key size={16} />
                        )}
                      </button>
                    )}
                    {account.status === 'registered' && account.kiro_refresh_token && account.kiro_refresh_token.trim() !== '' && (
                      <button
                        className="action-button action-button-success"
                        onClick={() => handleExportSingleJson(account)}
                        title="复制 JSON 到剪贴板"
                      >
                        <Copy size={16} />
                      </button>
                    )}
                    <button
                      className="action-button action-button-danger"
                      onClick={() => handleDelete(account.id)}
                      title="删除"
                    >
                      <Trash2 size={16} />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {totalPages > 1 && (
        <div className="pagination">
          <button
            onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
            disabled={currentPage === 1}
            className="pagination-button"
          >
            上一页
          </button>
          <span className="pagination-info">
            第 {currentPage} / {totalPages} 页
          </span>
          <button
            onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
            disabled={currentPage === totalPages}
            className="pagination-button"
          >
            下一页
          </button>
        </div>
      )}

      {isDetailModalOpen && selectedAccount && (
        <DetailModal
          account={selectedAccount}
          onClose={() => {
            setIsDetailModalOpen(false);
            setSelectedAccount(null);
          }}
        />
      )}

      {isEditModalOpen && selectedAccount && (
        <EditModal
          account={selectedAccount}
          onClose={() => {
            setIsEditModalOpen(false);
            setSelectedAccount(null);
          }}
          onSave={() => {
            setIsEditModalOpen(false);
            setSelectedAccount(null);
            onRefresh();
          }}
        />
      )}
    </div>
  );
}

function DetailModal({ account, onClose }: { account: Account; onClose: () => void }) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>账号详情</h2>
          <button className="modal-close" onClick={onClose}>×</button>
        </div>
        <div className="modal-body">
          <div className="detail-item">
            <label>ID:</label>
            <span>{account.id}</span>
          </div>
          <div className="detail-item">
            <label>注册邮箱:</label>
            <span>{account.email}</span>
          </div>
          <div className="detail-item">
            <label>邮箱密码:</label>
            <span>{account.email_password}</span>
          </div>
          <div className="detail-item">
            <label>客户端ID:</label>
            <span className="monospace">{account.client_id}</span>
          </div>
          <div className="detail-item">
            <label>Refresh Token:</label>
            <span className="monospace break-all">{account.refresh_token}</span>
          </div>
          {account.kiro_password && (
            <div className="detail-item">
              <label>Kiro密码:</label>
              <span>{account.kiro_password}</span>
            </div>
          )}
          <div className="detail-item">
            <label>状态:</label>
            <span>{account.status}</span>
          </div>
          {account.error_reason && (
            <div className="detail-item">
              <label>异常原因:</label>
              <span className="error-text">{account.error_reason}</span>
            </div>
          )}
          <div className="detail-item">
            <label>创建时间:</label>
            <span>{new Date(account.created_at).toLocaleString('zh-CN')}</span>
          </div>
          <div className="detail-item">
            <label>更新时间:</label>
            <span>{new Date(account.updated_at).toLocaleString('zh-CN')}</span>
          </div>
        </div>
      </div>
    </div>
  );
}

function EditModal({
  account,
  onClose,
  onSave,
}: {
  account: Account;
  onClose: () => void;
  onSave: () => void;
}) {
  const [formData, setFormData] = useState({
    email: account.email,
    email_password: account.email_password,
    client_id: account.client_id,
    refresh_token: account.refresh_token,
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await api.updateAccount({
        id: account.id,
        ...formData,
      });
      await showSuccess('更新成功');
      onSave();
    } catch (error) {
      await showError('更新失败: ' + error);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>编辑账号</h2>
          <button className="modal-close" onClick={onClose}>×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="modal-body">
            <div className="form-group">
              <label>注册邮箱:</label>
              <input
                type="email"
                value={formData.email}
                onChange={e => setFormData({ ...formData, email: e.target.value })}
                required
              />
            </div>
            <div className="form-group">
              <label>邮箱密码:</label>
              <input
                type="text"
                value={formData.email_password}
                onChange={e => setFormData({ ...formData, email_password: e.target.value })}
                required
              />
            </div>
            <div className="form-group">
              <label>客户端ID:</label>
              <input
                type="text"
                value={formData.client_id}
                onChange={e => setFormData({ ...formData, client_id: e.target.value })}
                required
              />
            </div>
            <div className="form-group">
              <label>Refresh Token:</label>
              <textarea
                value={formData.refresh_token}
                onChange={e => setFormData({ ...formData, refresh_token: e.target.value })}
                required
                rows={3}
              />
            </div>
          </div>
          <div className="modal-footer">
            <button type="button" onClick={onClose} className="button-secondary">
              取消
            </button>
            <button type="submit" className="button-primary">
              保存
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
