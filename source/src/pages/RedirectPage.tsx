import { useState, useCallback, useEffect } from 'react'
import { syntaxHighlight } from '../utils.ts'

const BASE_URL = window.location.origin + '/app-redirector'

interface RedirectRule {
  id: string
  path: string
  redirectURL: string
  statusCode: number
  host?: string
  version?: number
  regex?: boolean
  description?: string
}

export function RedirectPage() {
  const [rules, setRules] = useState<RedirectRule[]>([])
  const [loading, setLoading] = useState(false)
  const [checkPath, setCheckPath] = useState('/old-page')
  const [checkResult, setCheckResult] = useState('')
  const [checkLoading, setCheckLoading] = useState(false)
  const [metrics, setMetrics] = useState('')
  const [metricsLoading, setMetricsLoading] = useState(false)

  const fetchRules = useCallback(async () => {
    setLoading(true)
    try {
      const response = await fetch(`${BASE_URL}/rule/?limit=50`)
      if (response.ok) {
        const data = await response.json()
        setRules(Array.isArray(data) ? data : [])
      }
    } catch { /* ignore */ }
    finally { setLoading(false) }
  }, [])

  useEffect(() => { fetchRules() }, [fetchRules])

  const handleCheck = useCallback(async () => {
    if (!checkPath.trim()) return
    setCheckLoading(true)
    setCheckResult('')
    try {
      const response = await fetch(`${BASE_URL}/checkredirect?url=${encodeURIComponent(checkPath.trim())}`)
      const data = await response.json()
      setCheckResult(JSON.stringify(data, null, 2))
    } catch (err) {
      setCheckResult(err instanceof Error ? err.message : 'Error')
    } finally {
      setCheckLoading(false)
    }
  }, [checkPath])

  const handleMetrics = useCallback(async () => {
    setMetricsLoading(true)
    setMetrics('')
    try {
      const response = await fetch(`${BASE_URL}/redirectmetrics`)
      const data = await response.json()
      setMetrics(JSON.stringify(data, null, 2))
    } catch (err) {
      setMetrics(err instanceof Error ? err.message : 'Error')
    } finally {
      setMetricsLoading(false)
    }
  }, [])

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter') handleCheck()
  }, [handleCheck])

  return (
    <>
      {/* Left panel: Rules list */}
      <div className="panel">
        <div className="panel-header">
          <span className="panel-title">Redirect Rules</span>
          <span className="panel-badge">
            {loading ? '...' : `${rules.length} rules`}
          </span>
        </div>
        <div className="panel-body">
          {rules.length === 0 ? (
            <div className="empty-state">
              <p>No redirect rules loaded</p>
              <p style={{ fontSize: '0.7rem', marginTop: '0.5rem', color: '#666' }}>
                Rules are loaded from seed data on server start
              </p>
            </div>
          ) : (
            rules.map(rule => (
              <div key={rule.id} className="message-item">
                <div className="message-title">
                  {rule.path} → {rule.redirectURL}
                </div>
                <div className="message-content">
                  {rule.statusCode}
                  {rule.host && ` | Host: ${rule.host}`}
                  {rule.description && ` | ${rule.description}`}
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Right panel: Check + Metrics */}
      <div className="panel">
        <div className="panel-header">
          <span className="panel-title">Check Redirect</span>
          <div className="header-actions">
            <input
              type="text"
              className="search-input"
              placeholder="Path to check..."
              value={checkPath}
              onChange={e => setCheckPath(e.target.value)}
              onKeyDown={handleKeyDown}
            />
            <button
              className="btn btn-primary btn-sm"
              onClick={handleCheck}
              disabled={checkLoading || !checkPath.trim()}
            >
              {checkLoading ? 'Checking...' : 'Check'}
            </button>
          </div>
        </div>
        <div className="panel-body" style={{ flex: 1, overflow: 'auto', padding: 0 }}>
          {checkResult ? (
            <pre
              className="results-pre"
              style={{ padding: '1rem' }}
              dangerouslySetInnerHTML={{ __html: syntaxHighlight(checkResult) }}
            />
          ) : (
            <div className="empty-state">
              <p>Enter a path and click Check to test redirect rules</p>
            </div>
          )}
        </div>

        <div className="panel-header">
          <span className="panel-title">Metrics</span>
          <button
            className="btn btn-sm"
            onClick={handleMetrics}
            disabled={metricsLoading}
          >
            {metricsLoading ? 'Loading...' : 'Fetch Metrics'}
          </button>
        </div>
        <div className="panel-body" style={{ flex: 1, overflow: 'auto', padding: 0 }}>
          {metrics ? (
            <pre
              className="results-pre"
              style={{ padding: '1rem' }}
              dangerouslySetInnerHTML={{ __html: syntaxHighlight(metrics) }}
            />
          ) : (
            <div className="empty-state">
              <p>Click Fetch Metrics to view redirect analytics</p>
            </div>
          )}
        </div>
      </div>
    </>
  )
}
