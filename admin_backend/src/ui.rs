pub const INDEX_HTML: &str = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>KeyLessPass Admin</title>
  <style>
    :root {
      color-scheme: light;
      --bg: #f4f3ee;
      --paper: #fffdf7;
      --paper-soft: #faf8f1;
      --panel: #ffffff;
      --ink: #1d1c18;
      --muted: #706f67;
      --muted-2: #9a978b;
      --line: #dedbd0;
      --line-strong: #c7c2b5;
      --accent: #5e4bd8;
      --accent-ink: #ffffff;
      --accent-soft: #eeeaff;
      --blue: #2563eb;
      --green: #157f55;
      --green-soft: #e8f6ee;
      --red: #c43d3d;
      --red-soft: #fff0ee;
      --amber: #a8660f;
      --amber-soft: #fff4d6;
      --shadow: 0 18px 60px rgba(40, 38, 31, 0.10);
      --radius: 8px;
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }
    * { box-sizing: border-box; }
    html { scroll-behavior: smooth; }
    body {
      margin: 0;
      background:
        linear-gradient(180deg, rgba(255, 253, 247, 0.94), rgba(244, 243, 238, 0.98)),
        radial-gradient(circle at top left, rgba(94, 75, 216, 0.12), transparent 34%),
        var(--bg);
      color: var(--ink);
      min-height: 100vh;
    }
    button, input, textarea, select { font: inherit; letter-spacing: 0; }
    button {
      min-height: 38px;
      border: 1px solid transparent;
      border-radius: 7px;
      padding: 9px 13px;
      color: var(--accent-ink);
      background: var(--accent);
      font-weight: 760;
      cursor: pointer;
      transition: transform 120ms ease, box-shadow 120ms ease, background 120ms ease, border-color 120ms ease;
    }
    button:hover { transform: translateY(-1px); box-shadow: 0 8px 22px rgba(94, 75, 216, 0.18); }
    button:active { transform: translateY(0); box-shadow: none; }
    button.secondary {
      background: var(--paper);
      color: var(--ink);
      border-color: var(--line);
    }
    button.secondary:hover { box-shadow: 0 8px 22px rgba(40, 38, 31, 0.08); border-color: var(--line-strong); }
    button.ghost {
      background: transparent;
      color: var(--muted);
      border-color: transparent;
    }
    button.warning { background: var(--red); }
    button:disabled { opacity: .45; cursor: not-allowed; transform: none; box-shadow: none; }
    input, textarea, select {
      width: 100%;
      min-height: 40px;
      border: 1px solid var(--line);
      border-radius: 7px;
      background: #fffefa;
      color: var(--ink);
      padding: 10px 12px;
      outline: none;
    }
    input:focus, textarea:focus, select:focus {
      border-color: var(--accent);
      box-shadow: 0 0 0 3px rgba(94, 75, 216, 0.14);
    }
    textarea {
      min-height: 156px;
      resize: vertical;
      font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
      font-size: 12px;
      line-height: 1.55;
    }
    label {
      display: block;
      margin: 14px 0 6px;
      color: var(--muted);
      font-size: 12px;
      font-weight: 760;
    }
    h1, h2, h3, p { margin: 0; letter-spacing: 0; }
    h1 { font-size: clamp(30px, 5vw, 56px); line-height: .96; font-weight: 760; max-width: 760px; }
    h2 { font-size: 18px; line-height: 1.2; font-weight: 760; }
    h3 { font-size: 14px; line-height: 1.25; font-weight: 760; }
    p { color: var(--muted); line-height: 1.55; }
    code, .mono {
      font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
      font-size: 12px;
      word-break: break-all;
    }
    code {
      background: var(--paper-soft);
      border: 1px solid var(--line);
      border-radius: 5px;
      padding: 2px 5px;
    }
    .app-shell {
      display: grid;
      grid-template-columns: 260px minmax(0, 1fr);
      min-height: 100vh;
    }
    .sidebar {
      position: sticky;
      top: 0;
      height: 100vh;
      padding: 22px 18px;
      border-right: 1px solid var(--line);
      background: rgba(255, 253, 247, 0.72);
      backdrop-filter: blur(18px);
      display: flex;
      flex-direction: column;
      gap: 22px;
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 11px;
      min-height: 44px;
    }
    .brand-mark {
      width: 38px;
      height: 38px;
      border-radius: 8px;
      display: grid;
      place-items: center;
      background: var(--accent);
      color: white;
      font-weight: 860;
      box-shadow: 0 12px 30px rgba(94, 75, 216, 0.22);
    }
    .brand-title { font-weight: 820; }
    .brand-subtitle { color: var(--muted); font-size: 12px; margin-top: 2px; }
    .nav {
      display: grid;
      gap: 4px;
    }
    .nav a {
      color: var(--muted);
      text-decoration: none;
      font-size: 14px;
      font-weight: 720;
      padding: 10px 12px;
      border-radius: 7px;
      display: flex;
      align-items: center;
      gap: 9px;
    }
    .nav a:hover, .nav a.active {
      color: var(--ink);
      background: var(--paper);
      box-shadow: inset 0 0 0 1px var(--line);
    }
    .nav-dot {
      width: 7px;
      height: 7px;
      border-radius: 99px;
      background: var(--muted-2);
      flex: 0 0 auto;
    }
    .nav a.active .nav-dot { background: var(--accent); }
    .sidebar-footer {
      margin-top: auto;
      display: grid;
      gap: 10px;
    }
    .status-pill {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      min-height: 34px;
      padding: 7px 10px;
      border-radius: 999px;
      border: 1px solid var(--line);
      background: var(--paper);
      color: var(--muted);
      font-size: 12px;
      font-weight: 760;
      max-width: 100%;
    }
    .status-dot {
      width: 8px;
      height: 8px;
      border-radius: 99px;
      background: var(--muted-2);
      flex: 0 0 auto;
    }
    .status-pill.ok .status-dot { background: var(--green); }
    .status-pill.bad .status-dot { background: var(--red); }
    .content {
      min-width: 0;
      padding: 30px clamp(20px, 4vw, 48px) 52px;
    }
    .hero {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 24px;
      align-items: start;
      margin: 10px 0 26px;
    }
    .eyebrow {
      color: var(--accent);
      font-size: 12px;
      font-weight: 820;
      text-transform: uppercase;
      letter-spacing: .08em;
      margin-bottom: 13px;
    }
    .hero-copy {
      max-width: 780px;
      margin-top: 14px;
      font-size: 15px;
    }
    .hero-actions {
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
      justify-content: flex-end;
    }
    .section {
      scroll-margin-top: 24px;
      margin-top: 18px;
    }
    .section-head {
      display: flex;
      align-items: end;
      justify-content: space-between;
      gap: 16px;
      margin: 0 0 12px;
    }
    .section-head p { font-size: 13px; margin-top: 5px; }
    .grid {
      display: grid;
      gap: 14px;
    }
    .grid.two { grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); align-items: start; }
    .grid.three { grid-template-columns: repeat(3, minmax(0, 1fr)); }
    .card {
      background: rgba(255, 255, 255, 0.82);
      border: 1px solid var(--line);
      border-radius: var(--radius);
      box-shadow: var(--shadow);
      padding: 18px;
      min-width: 0;
    }
    .card.subtle {
      box-shadow: none;
      background: rgba(255, 253, 247, 0.7);
    }
    .metric {
      min-height: 126px;
      display: grid;
      align-content: space-between;
    }
    .metric-label {
      color: var(--muted);
      font-size: 12px;
      font-weight: 760;
      text-transform: uppercase;
      letter-spacing: .06em;
    }
    .metric-value {
      margin-top: 16px;
      font-size: clamp(28px, 4vw, 40px);
      font-weight: 760;
      line-height: 1;
    }
    .metric-note { color: var(--muted); font-size: 12px; margin-top: 12px; }
    .toolbar {
      display: flex;
      gap: 10px;
      align-items: center;
      flex-wrap: wrap;
    }
    .toolbar input, .toolbar select { width: auto; min-width: 220px; }
    .split {
      display: grid;
      grid-template-columns: minmax(0, .92fr) minmax(0, 1.08fr);
      gap: 14px;
      align-items: start;
    }
    .form-grid {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 0 12px;
    }
    .form-grid .full { grid-column: 1 / -1; }
    .callout {
      padding: 12px;
      border: 1px solid var(--line);
      border-radius: 7px;
      background: var(--paper-soft);
      color: var(--muted);
      font-size: 13px;
      line-height: 1.5;
    }
    .callout.info { border-color: rgba(94, 75, 216, 0.22); background: var(--accent-soft); color: #3a3177; }
    .callout.warn { border-color: rgba(168, 102, 15, 0.26); background: var(--amber-soft); color: #6b430e; }
    .callout.danger { border-color: rgba(196, 61, 61, 0.28); background: var(--red-soft); color: #7c2929; }
    .key-box {
      display: grid;
      gap: 10px;
      margin-top: 12px;
    }
    .key-line {
      display: grid;
      grid-template-columns: 120px minmax(0, 1fr) auto;
      gap: 10px;
      align-items: center;
      padding: 10px;
      border: 1px solid var(--line);
      border-radius: 7px;
      background: #fffefa;
    }
    .key-line strong { color: var(--muted); font-size: 12px; }
    .table-wrap {
      overflow-x: auto;
      border: 1px solid var(--line);
      border-radius: var(--radius);
      background: rgba(255, 253, 247, 0.68);
    }
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 13px;
      min-width: 780px;
    }
    th, td {
      text-align: left;
      padding: 12px 12px;
      border-bottom: 1px solid var(--line);
      vertical-align: middle;
    }
    th {
      color: var(--muted);
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: .06em;
      background: rgba(250, 248, 241, 0.9);
    }
    tbody tr:last-child td { border-bottom: 0; }
    tbody tr:hover td { background: rgba(94, 75, 216, 0.035); }
    .check-cell { width: 44px; }
    input[type="checkbox"] {
      width: 16px;
      min-height: 16px;
      accent-color: var(--accent);
    }
    .badge {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      min-height: 24px;
      border-radius: 999px;
      padding: 4px 9px;
      font-size: 12px;
      font-weight: 780;
      background: var(--paper-soft);
      color: var(--muted);
      border: 1px solid var(--line);
      white-space: nowrap;
    }
    .badge.green { background: var(--green-soft); color: var(--green); border-color: rgba(21, 127, 85, 0.16); }
    .badge.red { background: var(--red-soft); color: var(--red); border-color: rgba(196, 61, 61, 0.18); }
    .badge.purple { background: var(--accent-soft); color: var(--accent); border-color: rgba(94, 75, 216, 0.18); }
    .empty {
      padding: 34px 18px;
      text-align: center;
      color: var(--muted);
    }
    .empty strong {
      display: block;
      color: var(--ink);
      margin-bottom: 6px;
    }
    .bundle-output {
      min-height: 210px;
      margin-top: 12px;
      background: #11100e;
      color: #f8f3e8;
      border-color: #11100e;
    }
    .toast {
      position: fixed;
      right: 24px;
      bottom: 24px;
      width: min(420px, calc(100vw - 32px));
      padding: 13px 14px;
      border-radius: 8px;
      border: 1px solid var(--line);
      background: rgba(255, 253, 247, 0.96);
      box-shadow: 0 20px 58px rgba(40, 38, 31, .16);
      color: var(--ink);
      transform: translateY(16px);
      opacity: 0;
      pointer-events: none;
      transition: opacity 160ms ease, transform 160ms ease;
      z-index: 20;
    }
    .toast.show { opacity: 1; transform: translateY(0); }
    .toast.bad { border-color: rgba(196, 61, 61, .28); background: #fff7f5; color: #7c2929; }
    .fine-print {
      color: var(--muted-2);
      font-size: 12px;
      line-height: 1.5;
    }
    .mobile-topbar { display: none; }
    @media (max-width: 1120px) {
      .app-shell { grid-template-columns: 1fr; }
      .sidebar { display: none; }
      .mobile-topbar {
        display: flex;
        position: sticky;
        top: 0;
        z-index: 5;
        justify-content: space-between;
        align-items: center;
        gap: 12px;
        padding: 14px 18px;
        border-bottom: 1px solid var(--line);
        background: rgba(255, 253, 247, .88);
        backdrop-filter: blur(18px);
      }
      .hero { grid-template-columns: 1fr; }
      .hero-actions { justify-content: flex-start; }
      .grid.three { grid-template-columns: 1fr; }
      .split, .grid.two { grid-template-columns: 1fr; }
    }
    @media (max-width: 680px) {
      .content { padding: 22px 14px 42px; }
      .form-grid { grid-template-columns: 1fr; }
      .toolbar input, .toolbar select { width: 100%; min-width: 0; }
      .key-line { grid-template-columns: 1fr; }
      button { width: 100%; }
      .hero-actions button, .toolbar button { width: auto; }
      .mobile-topbar button { width: auto; min-width: 120px; }
    }
  </style>
</head>
<body>
  <div class="mobile-topbar">
    <div class="brand">
      <div class="brand-mark">K</div>
      <div>
        <div class="brand-title">KeyLessPass Admin</div>
        <div class="brand-subtitle">License operations</div>
      </div>
    </div>
    <button class="secondary" id="mobileRefreshButton">Refresh</button>
  </div>

  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-mark">K</div>
        <div>
          <div class="brand-title">KeyLessPass</div>
          <div class="brand-subtitle">Commercial Admin</div>
        </div>
      </div>
      <nav class="nav" aria-label="Admin sections">
        <a href="#overview" class="active"><span class="nav-dot"></span>Overview</a>
        <a href="#operations"><span class="nav-dot"></span>Operations</a>
        <a href="#devices"><span class="nav-dot"></span>Devices</a>
        <a href="#history"><span class="nav-dot"></span>Bundles and grants</a>
        <a href="#security"><span class="nav-dot"></span>Security</a>
      </nav>
      <div class="sidebar-footer">
        <div class="status-pill" id="healthPill"><span class="status-dot"></span><span>Not connected</span></div>
        <button class="secondary" id="refreshButton">Refresh</button>
      </div>
    </aside>

    <main class="content">
      <section class="hero" id="overview">
        <div>
          <div class="eyebrow">Offline license authority</div>
          <h1>Issue device grants without touching password secrets.</h1>
          <p class="hero-copy">Manage enterprise seats, import desktop device requests, and ship signed license bundles for intranet KeyLessPass deployments.</p>
        </div>
        <div class="hero-actions">
          <button class="secondary" id="copyPublicKeyTopButton">Copy site public key</button>
          <button id="jumpIssueButton">Issue bundle</button>
        </div>
      </section>

      <section class="section">
        <div class="grid three">
          <div class="card metric">
            <div>
              <div class="metric-label">Organizations</div>
              <div class="metric-value" id="metricOrganizations">0</div>
            </div>
            <div class="metric-note">Active commercial accounts</div>
          </div>
          <div class="card metric">
            <div>
              <div class="metric-label">Devices</div>
              <div class="metric-value" id="metricDevices">0</div>
            </div>
            <div class="metric-note">Imported authorization requests</div>
          </div>
          <div class="card metric">
            <div>
              <div class="metric-label">Bundles</div>
              <div class="metric-value" id="metricBundles">0</div>
            </div>
            <div class="metric-note">Signed offline packages</div>
          </div>
        </div>
      </section>

      <section class="section grid two">
        <div class="card" id="tokenCard">
          <div class="section-head">
            <div>
              <h2>Admin access</h2>
              <p>Token protected local session.</p>
            </div>
            <span class="badge" id="tokenBadge">No token</span>
          </div>
          <label for="tokenInput">Admin token</label>
          <div class="toolbar">
            <input id="tokenInput" type="password" autocomplete="off" placeholder="Paste KEYLESSPASS_ADMIN_TOKEN">
            <button id="saveTokenButton">Connect</button>
            <button class="secondary" id="clearTokenButton">Forget</button>
          </div>
        </div>

        <div class="card" id="signingCard">
          <div class="section-head">
            <div>
              <h2>Customer-site signing key</h2>
              <p>Delegated by the vendor entitlement.</p>
            </div>
            <span class="badge purple" id="keyIdBadge">keylesspass-license-2026-q3</span>
          </div>
          <div class="callout warn">Do not embed this site key in clients. Send it to the vendor for entitlement delegation; commercial clients embed only the vendor root public key.</div>
          <div class="key-box" id="statusBox">
            <div class="empty"><strong>No status loaded</strong>Connect with the admin token.</div>
          </div>
        </div>
      </section>

      <section class="section" id="operations">
        <div class="section-head">
          <div>
            <h2>Operations</h2>
            <p>Create organizations, import devices, and issue signed bundles.</p>
          </div>
        </div>
        <div class="split">
          <div class="grid">
            <div class="card">
              <div class="section-head">
                <div>
                  <h2>Create organization</h2>
                  <p>Commercial account and entitlement envelope.</p>
                </div>
              </div>
              <div class="form-grid">
                <div class="full">
                  <label for="orgName">Organization name</label>
                  <input id="orgName" placeholder="Acme Railway Group">
                </div>
                <div>
                  <label for="orgPlan">Plan</label>
                  <input id="orgPlan" value="enterprise">
                </div>
                <div>
                  <label for="orgSeats">Max seats</label>
                  <input id="orgSeats" type="number" min="1" value="25">
                </div>
                <div>
                  <label for="orgDays">Valid days</label>
                  <input id="orgDays" type="number" min="1" value="365">
                </div>
                <div>
                  <label for="orgGrace">Offline grace days</label>
                  <input id="orgGrace" type="number" min="0" value="14">
                </div>
                <div class="full">
                  <label for="orgFeatures">Features</label>
                  <input id="orgFeatures" value="desktop-client, channel:commercial">
                </div>
              </div>
              <div class="toolbar" style="margin-top:14px">
                <button id="createOrgButton">Create organization</button>
              </div>
            </div>

            <div class="card">
              <div class="section-head">
                <div>
                  <h2>Import device request</h2>
                  <p>Assign a desktop installation to an organization.</p>
                </div>
              </div>
              <label for="deviceOrg">Organization</label>
              <select id="deviceOrg"></select>
              <label for="deviceSeat">Seat label</label>
              <input id="deviceSeat" placeholder="Finance laptop 01">
              <label for="deviceRequestJson">Request JSON</label>
              <textarea id="deviceRequestJson" placeholder='{"schemaVersion":2,...}'></textarea>
              <div class="toolbar" style="margin-top:14px">
                <button id="importDeviceButton">Import request</button>
              </div>
            </div>
          </div>

          <div class="card" id="issueCard">
            <div class="section-head">
              <div>
                <h2>Issue license bundle</h2>
                <p>Sign selected devices into an offline bundle.</p>
              </div>
              <span class="badge purple" id="selectionBadge">0 selected</span>
            </div>
            <label for="issueOrg">Organization</label>
            <select id="issueOrg"></select>
            <label for="issueDays">Valid days override</label>
            <input id="issueDays" type="number" min="1" placeholder="Use organization expiry">
            <div class="toolbar" style="margin-top:14px">
              <button class="secondary" id="selectOrgDevicesButton">Select organization devices</button>
              <button id="issueBundleButton">Issue bundle</button>
            </div>
            <textarea class="bundle-output" id="bundleOutput" readonly placeholder="Signed bundle JSON appears here."></textarea>
            <div class="toolbar" style="margin-top:12px">
              <button class="secondary" id="copyBundleButton">Copy bundle</button>
              <button class="secondary" id="downloadBundleButton">Download bundle</button>
            </div>
          </div>

          <div class="card">
            <div class="section-head">
              <div>
                <h2>Organization activation</h2>
                <p>Share activation codes only through an approved secure channel.</p>
              </div>
            </div>
            <div class="table-wrap">
              <table>
                <thead><tr><th>Organization</th><th>Seats</th><th>Activation code</th><th></th></tr></thead>
                <tbody id="organizationsTable"></tbody>
              </table>
            </div>
          </div>
        </div>
      </section>

      <section class="section" id="devices">
        <div class="section-head">
          <div>
            <h2>Devices</h2>
            <p>Imported commercial device identities.</p>
          </div>
          <div class="toolbar">
            <input id="deviceSearch" placeholder="Search seat, ID, fingerprint">
            <select id="deviceFilterOrg"></select>
            <input id="deviceCsvFile" type="file" accept=".csv,text/csv">
            <button class="secondary" id="importDeviceCsvButton">Import CSV</button>
            <button class="secondary" id="exportDeviceCsvButton">Export CSV</button>
          </div>
        </div>
        <div class="card">
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th class="check-cell"></th>
                  <th>Seat</th>
                  <th>Organization</th>
                  <th>Commercial device ID</th>
                  <th>Fingerprint</th>
                  <th>Platform</th>
                  <th>Updated</th>
                </tr>
              </thead>
              <tbody id="devicesTable"></tbody>
            </table>
          </div>
        </div>
      </section>

      <section class="section" id="history">
        <div class="section-head">
          <div>
            <h2>Bundles and grants</h2>
            <p>Signed history and active revocation records.</p>
          </div>
        </div>
        <div class="grid two">
          <div class="card">
            <div class="section-head">
              <h2>Issued bundles</h2>
              <span class="badge" id="bundleCountBadge">0 bundles</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead><tr><th>Bundle</th><th>Devices</th><th>Issued</th><th></th></tr></thead>
                <tbody id="bundlesTable"></tbody>
              </table>
            </div>
          </div>
          <div class="card">
            <div class="section-head">
              <h2>Grants</h2>
              <span class="badge" id="grantCountBadge">0 grants</span>
            </div>
            <div class="table-wrap">
              <table>
                <thead><tr><th>Grant</th><th>Seat</th><th>Status</th><th></th></tr></thead>
                <tbody id="grantsTable"></tbody>
              </table>
            </div>
          </div>
        </div>
        <div class="card" style="margin-top:18px">
          <div class="section-head">
            <div>
              <h2>Audit log</h2>
              <p>Latest authorization administration events.</p>
            </div>
            <button class="secondary" id="exportAuditCsvButton">Export audit CSV</button>
          </div>
          <div class="table-wrap">
            <table>
              <thead><tr><th>Time</th><th>Actor</th><th>Role</th><th>Action</th><th>Target</th></tr></thead>
              <tbody id="auditTable"></tbody>
            </table>
          </div>
        </div>
      </section>

      <section class="section" id="security">
        <div class="card subtle">
          <div class="section-head">
            <div>
              <h2>Security boundary</h2>
              <p>Commercial authorization is separate from password recovery and derivation.</p>
            </div>
          </div>
          <div class="grid two">
            <div class="callout danger">Do not paste mnemonic phrases, Kmaster, deviceSecret, usbSecret, CDR secrets, service passwords, derived passwords, or wrapper keys into this backend.</div>
            <div class="callout warn">Grant revocation affects commercial authorization only. It must not delete local factor packages, USB packages, CDRs, or recovery wrappers.</div>
          </div>
          <p class="fine-print" style="margin-top:14px">The backend signs licensing metadata. The KeyLessPass password security boundary remains local to the desktop client and its factors.</p>
        </div>
      </section>
    </main>
  </div>

  <div class="toast" id="toast"></div>

  <script>
    const state = {
      snapshot: null,
      selectedDeviceIds: new Set(),
      lastBundle: "",
      deviceSearch: "",
      deviceFilterOrg: "all",
      toastTimer: null,
    };
    const $ = (id) => document.getElementById(id);

    function token() { return sessionStorage.getItem("klpAdminToken") || ""; }
    function headers() {
      return { "content-type": "application/json", "authorization": `Bearer ${token()}` };
    }
    async function api(path, options = {}) {
      const res = await fetch(path, { ...options, headers: { ...headers(), ...(options.headers || {}) } });
      const text = await res.text();
      const data = text ? JSON.parse(text) : {};
      if (!res.ok) throw new Error(data.error || data.message || res.statusText);
      return data;
    }
    function showToast(message, bad = false) {
      const toast = $("toast");
      toast.textContent = message;
      toast.className = `toast show${bad ? " bad" : ""}`;
      clearTimeout(state.toastTimer);
      state.toastTimer = setTimeout(() => { toast.className = "toast"; }, 3200);
    }
    function setHealth(text, ok) {
      const pill = $("healthPill");
      pill.className = `status-pill ${ok ? "ok" : "bad"}`;
      pill.innerHTML = `<span class="status-dot"></span><span>${escapeHtml(text)}</span>`;
    }
    function featuresFromInput(value) {
      return value.split(",").map((v) => v.trim()).filter(Boolean);
    }
    function setButtonBusy(button, busy, label) {
      if (!button) return;
      if (busy) {
        button.dataset.originalText = button.textContent;
        button.textContent = label || "Working";
        button.disabled = true;
      } else {
        button.textContent = button.dataset.originalText || button.textContent;
        button.disabled = false;
      }
    }
    async function withBusy(button, label, fn) {
      setButtonBusy(button, true, label);
      try {
        return await fn();
      } finally {
        setButtonBusy(button, false);
      }
    }
    async function refresh() {
      try {
        state.snapshot = await api("/api/snapshot");
        render();
        setHealth("Connected", true);
        showToast("Dashboard refreshed.");
      } catch (err) {
        setHealth("Connection failed", false);
        showToast(err.message, true);
      }
    }
    function render() {
      const snap = state.snapshot;
      if (!snap) {
        renderTokenState();
        return;
      }
      renderTokenState();
      $("metricOrganizations").textContent = snap.status.organizationCount;
      $("metricDevices").textContent = snap.status.deviceCount;
      $("metricBundles").textContent = snap.status.bundleCount;
      $("keyIdBadge").textContent = snap.status.keyId;
      renderStatusBox(snap.status);
      renderOrgSelects(snap.organizations);
      renderOrganizations(snap.organizations);
      renderDevices(snap.devices);
      renderBundles(snap.bundles);
      renderGrants(snap.grants);
      renderAudit(snap.auditLog || []);
      renderActions();
    }
    function renderTokenState() {
      const hasToken = Boolean(token());
      $("tokenBadge").textContent = hasToken ? "Stored locally" : "No token";
      $("tokenBadge").className = `badge ${hasToken ? "green" : ""}`;
    }
    function renderStatusBox(status) {
      $("statusBox").innerHTML = [
        keyLine("customer", status.customerId, false),
        keyLine("entitlement serial", String(status.entitlementSerial), false),
        keyLine("approved devices", `${status.approvedDeviceCount} / ${status.maxRegisteredDevices}`, false),
        keyLine("entitlement expires", status.entitlementValidUntil, false),
        keyLine("keyId", status.keyId, false),
        keyLine("publicKeyB64", status.publicKeyB64, true, "publicKeyB64"),
        keyLine("publicKeyB64Url", status.publicKeyB64url, true, "publicKeyB64url"),
        keyLine("database", status.databasePath, false),
      ].join("");
      $("statusBox").querySelectorAll("button[data-copy]").forEach((button) => {
        button.addEventListener("click", async () => {
          await copyText(button.dataset.copy || "");
          showToast("Public key copied.");
        });
      });
    }
    function keyLine(label, value, copyable, key) {
      return `
        <div class="key-line">
          <strong>${escapeHtml(label)}</strong>
          <span class="mono">${escapeHtml(value || "")}</span>
          ${copyable ? `<button class="secondary" data-copy="${escapeAttr(value || "")}" data-key="${escapeAttr(key || "")}">Copy</button>` : ""}
        </div>
      `;
    }
    function renderOrgSelects(orgs) {
      const options = orgs.map((org) => `<option value="${escapeAttr(org.id)}">${escapeHtml(org.name)} (${escapeHtml(org.id)})</option>`).join("");
      for (const id of ["deviceOrg", "issueOrg"]) {
        const selected = $(id).value;
        $(id).innerHTML = options || `<option value="">Create an organization first</option>`;
        if (selected && orgs.some((org) => org.id === selected)) $(id).value = selected;
      }
      const filterSelected = $("deviceFilterOrg").value || state.deviceFilterOrg;
      $("deviceFilterOrg").innerHTML = `<option value="all">All organizations</option>${options}`;
      if ([...$("deviceFilterOrg").options].some((option) => option.value === filterSelected)) {
        $("deviceFilterOrg").value = filterSelected;
      }
      state.deviceFilterOrg = $("deviceFilterOrg").value || "all";
    }
    function renderOrganizations(orgs) {
      if (!orgs.length) {
        $("organizationsTable").innerHTML = emptyRow(4, "No organizations", "Create an organization to receive an activation code.");
        return;
      }
      $("organizationsTable").innerHTML = orgs.map((org) => `
        <tr>
          <td><strong>${escapeHtml(org.name)}</strong><div class="fine-print mono">${escapeHtml(org.id)}</div></td>
          <td>${org.maxSeats}</td>
          <td class="mono">${escapeHtml(shorten(org.activationCode, 34))}</td>
          <td><button class="secondary" data-activation-code="${escapeAttr(org.activationCode)}">Copy</button></td>
        </tr>
      `).join("");
      $("organizationsTable").querySelectorAll("button").forEach((button) => {
        button.addEventListener("click", async () => {
          await copyText(button.dataset.activationCode || "");
          showToast("Activation code copied.");
        });
      });
    }
    function filteredDevices(devices) {
      const query = state.deviceSearch.trim().toLowerCase();
      return devices.filter((device) => {
        const orgMatch = state.deviceFilterOrg === "all" || device.organizationId === state.deviceFilterOrg;
        if (!orgMatch) return false;
        if (!query) return true;
        return [
          device.seatLabel,
          device.organizationId,
          device.commercialDeviceId,
          device.deviceFingerprint,
          device.platform,
          device.appVersion,
          device.buildChannel,
        ].join(" ").toLowerCase().includes(query);
      });
    }
    function renderDevices(devices) {
      const visible = filteredDevices(devices);
      if (visible.length === 0) {
        $("devicesTable").innerHTML = emptyRow(7, "No matching devices", devices.length ? "Adjust search or organization filter." : "Import a desktop authorization request.");
        updateSelectionBadge();
        return;
      }
      $("devicesTable").innerHTML = visible.map((device) => `
        <tr>
          <td class="check-cell"><input type="checkbox" data-device-id="${escapeAttr(device.id)}" ${state.selectedDeviceIds.has(device.id) ? "checked" : ""}></td>
          <td><strong>${escapeHtml(device.seatLabel || "Unlabeled seat")}</strong></td>
          <td><span class="badge">${escapeHtml(device.organizationId)}</span></td>
          <td class="mono">${escapeHtml(device.commercialDeviceId)}</td>
          <td class="mono">${escapeHtml(shorten(device.deviceFingerprint, 28))}</td>
          <td>${escapeHtml(device.platform)} <span class="fine-print">${escapeHtml(device.appVersion)}</span></td>
          <td>${formatDate(device.updatedAt)}</td>
        </tr>
      `).join("");
      $("devicesTable").querySelectorAll("input[type=checkbox]").forEach((box) => {
        box.addEventListener("change", () => {
          if (box.checked) state.selectedDeviceIds.add(box.dataset.deviceId);
          else state.selectedDeviceIds.delete(box.dataset.deviceId);
          updateSelectionBadge();
          renderActions();
        });
      });
      updateSelectionBadge();
    }
    function renderBundles(bundles) {
      $("bundleCountBadge").textContent = `${bundles.length} bundles`;
      if (bundles.length === 0) {
        $("bundlesTable").innerHTML = emptyRow(4, "No bundles issued", "Select devices and issue the first signed bundle.");
        return;
      }
      $("bundlesTable").innerHTML = bundles.map((bundle) => `
        <tr>
          <td class="mono">${escapeHtml(shorten(bundle.bundleId, 30))}</td>
          <td>${bundle.deviceCount}</td>
          <td>${formatDate(bundle.issuedAt)}</td>
          <td><button class="secondary" data-bundle-id="${escapeAttr(bundle.bundleId)}">Load</button></td>
        </tr>
      `).join("");
      $("bundlesTable").querySelectorAll("button").forEach((button) => {
        button.addEventListener("click", () => {
          const bundle = state.snapshot.bundles.find((item) => item.bundleId === button.dataset.bundleId);
          if (bundle) {
            setBundleOutput(bundle.envelopeJson);
            showToast("Bundle loaded.");
          }
        });
      });
    }
    function renderGrants(grants) {
      $("grantCountBadge").textContent = `${grants.length} grants`;
      if (grants.length === 0) {
        $("grantsTable").innerHTML = emptyRow(4, "No grants yet", "Issued bundles create device grants.");
        return;
      }
      $("grantsTable").innerHTML = grants.map((grant) => `
        <tr>
          <td class="mono">${escapeHtml(shorten(grant.grantId, 30))}</td>
          <td>${escapeHtml(grant.seatLabel || "Unlabeled seat")}</td>
          <td>${grant.revoked ? '<span class="badge red">Revoked</span>' : '<span class="badge green">Active</span>'}</td>
          <td>${grant.revoked ? "" : `<button class="warning" data-grant-id="${escapeAttr(grant.grantId)}">Revoke</button>`}</td>
        </tr>
      `).join("");
      $("grantsTable").querySelectorAll("button").forEach((button) => {
        button.addEventListener("click", async () => {
          if (!confirm("Revoke this device grant? Issue a new bundle afterward so clients can import the revocation list.")) return;
          await withBusy(button, "Revoking", async () => {
            await api(`/api/grants/${encodeURIComponent(button.dataset.grantId)}/revoke`, { method: "POST", body: "{}" });
            await refresh();
            showToast("Grant revoked.");
          });
        });
      });
    }
    function renderAudit(records) {
      if (!records.length) {
        $("auditTable").innerHTML = emptyRow(5, "No audit events", "Administrative changes will appear here.");
        return;
      }
      $("auditTable").innerHTML = records.slice(0, 100).map((record) => `
        <tr>
          <td>${formatDate(record.createdAt)}</td>
          <td>${escapeHtml(record.actor)}</td>
          <td><span class="badge">${escapeHtml(record.role)}</span></td>
          <td>${escapeHtml(record.action)}</td>
          <td class="mono">${escapeHtml(shorten(record.target, 36))}</td>
        </tr>
      `).join("");
    }
    function renderActions() {
      const hasOrg = Boolean(state.snapshot?.organizations?.length);
      $("importDeviceButton").disabled = !hasOrg;
      $("selectOrgDevicesButton").disabled = !hasOrg;
      $("issueBundleButton").disabled = !hasOrg || state.selectedDeviceIds.size === 0;
      $("copyPublicKeyTopButton").disabled = !state.snapshot?.status?.publicKeyB64;
    }
    function updateSelectionBadge() {
      $("selectionBadge").textContent = `${state.selectedDeviceIds.size} selected`;
    }
    function setBundleOutput(value) {
      state.lastBundle = value;
      $("bundleOutput").value = value;
    }
    function emptyRow(cols, title, note) {
      return `<tr><td colspan="${cols}"><div class="empty"><strong>${escapeHtml(title)}</strong>${escapeHtml(note)}</div></td></tr>`;
    }
    function shorten(value, length) {
      const text = String(value || "");
      if (text.length <= length) return text;
      return `${text.slice(0, Math.max(0, length - 7))}...${text.slice(-4)}`;
    }
    function formatDate(value) {
      if (!value) return "";
      const date = new Date(value);
      if (Number.isNaN(date.getTime())) return escapeHtml(value);
      return date.toLocaleString(undefined, { year: "numeric", month: "short", day: "2-digit", hour: "2-digit", minute: "2-digit" });
    }
    async function copyText(value) {
      if (!value) throw new Error("Nothing to copy.");
      await navigator.clipboard.writeText(value);
    }
    async function downloadApi(path, filename) {
      const res = await fetch(path, { headers: { authorization: `Bearer ${token()}` } });
      if (!res.ok) {
        const text = await res.text();
        throw new Error(text || res.statusText);
      }
      const blob = await res.blob();
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = filename;
      link.click();
      URL.revokeObjectURL(url);
    }
    function escapeHtml(value) {
      return String(value ?? "").replace(/[&<>"']/g, (ch) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[ch]));
    }
    function escapeAttr(value) { return escapeHtml(value).replace(/`/g, "&#96;"); }

    $("tokenInput").value = token();
    renderTokenState();

    $("saveTokenButton").addEventListener("click", async () => {
      sessionStorage.setItem("klpAdminToken", $("tokenInput").value.trim());
      await withBusy($("saveTokenButton"), "Connecting", refresh);
    });
    $("clearTokenButton").addEventListener("click", () => {
      sessionStorage.removeItem("klpAdminToken");
      $("tokenInput").value = "";
      state.snapshot = null;
      render();
      setHealth("Not connected", false);
      showToast("Admin token forgotten.");
    });
    $("refreshButton").addEventListener("click", () => withBusy($("refreshButton"), "Refreshing", refresh));
    $("mobileRefreshButton").addEventListener("click", () => withBusy($("mobileRefreshButton"), "Refreshing", refresh));
    $("jumpIssueButton").addEventListener("click", () => $("issueCard").scrollIntoView({ behavior: "smooth", block: "start" }));
    $("copyPublicKeyTopButton").addEventListener("click", async () => {
      try {
        await copyText(state.snapshot?.status?.publicKeyB64 || "");
        showToast("Public key copied.");
      } catch (err) {
        showToast(err.message, true);
      }
    });
    $("createOrgButton").addEventListener("click", async () => {
      await withBusy($("createOrgButton"), "Creating", async () => {
        await api("/api/organizations", {
          method: "POST",
          body: JSON.stringify({
            name: $("orgName").value,
            plan: $("orgPlan").value,
            maxSeats: Number($("orgSeats").value || 25),
            validDays: Number($("orgDays").value || 365),
            features: featuresFromInput($("orgFeatures").value),
            offlineGraceDays: Number($("orgGrace").value || 14),
          }),
        });
        $("orgName").value = "";
        await refresh();
        showToast("Organization created.");
      });
    });
    $("importDeviceButton").addEventListener("click", async () => {
      await withBusy($("importDeviceButton"), "Importing", async () => {
        await api("/api/device-requests/import", {
          method: "POST",
          body: JSON.stringify({
            organizationId: $("deviceOrg").value,
            seatLabel: $("deviceSeat").value,
            requestJson: $("deviceRequestJson").value,
          }),
        });
        $("deviceSeat").value = "";
        $("deviceRequestJson").value = "";
        await refresh();
        showToast("Device request imported.");
      });
    });
    $("importDeviceCsvButton").addEventListener("click", async () => {
      const file = $("deviceCsvFile").files[0];
      if (!file) {
        showToast("Choose a CSV file first.", true);
        return;
      }
      await withBusy($("importDeviceCsvButton"), "Importing", async () => {
        const result = await api("/api/device-requests/import.csv", {
          method: "POST",
          headers: { "content-type": "text/csv" },
          body: await file.text(),
        });
        $("deviceCsvFile").value = "";
        await refresh();
        showToast(`${result.imported} device requests imported.`);
      });
    });
    $("exportDeviceCsvButton").addEventListener("click", async () => {
      try {
        await downloadApi("/api/devices.csv", "keylesspass-devices.csv");
        showToast("Device CSV downloaded.");
      } catch (err) {
        showToast(err.message, true);
      }
    });
    $("exportAuditCsvButton").addEventListener("click", async () => {
      try {
        await downloadApi("/api/audit.csv", "keylesspass-audit.csv");
        showToast("Audit CSV downloaded.");
      } catch (err) {
        showToast(err.message, true);
      }
    });
    $("selectOrgDevicesButton").addEventListener("click", () => {
      const orgId = $("issueOrg").value;
      state.selectedDeviceIds = new Set((state.snapshot?.devices || []).filter((device) => device.organizationId === orgId).map((device) => device.id));
      renderDevices(state.snapshot?.devices || []);
      renderActions();
      showToast(`${state.selectedDeviceIds.size} devices selected.`);
    });
    $("issueBundleButton").addEventListener("click", async () => {
      await withBusy($("issueBundleButton"), "Signing", async () => {
        const payload = {
          organizationId: $("issueOrg").value,
          deviceIds: Array.from(state.selectedDeviceIds),
        };
        if ($("issueDays").value) payload.validDays = Number($("issueDays").value);
        const issued = await api("/api/licenses/issue", { method: "POST", body: JSON.stringify(payload) });
        setBundleOutput(issued.envelopeJson);
        await refresh();
        setBundleOutput(issued.envelopeJson);
        showToast("Signed bundle issued.");
      });
    });
    $("copyBundleButton").addEventListener("click", async () => {
      try {
        await copyText($("bundleOutput").value);
        showToast("Bundle copied.");
      } catch (err) {
        showToast(err.message, true);
      }
    });
    $("downloadBundleButton").addEventListener("click", () => {
      const value = $("bundleOutput").value;
      if (!value) {
        showToast("No bundle to download.", true);
        return;
      }
      const blob = new Blob([value], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = "keylesspass.klp-license-bundle";
      link.click();
      URL.revokeObjectURL(url);
      showToast("Bundle download started.");
    });
    $("deviceSearch").addEventListener("input", () => {
      state.deviceSearch = $("deviceSearch").value;
      renderDevices(state.snapshot?.devices || []);
    });
    $("deviceFilterOrg").addEventListener("change", () => {
      state.deviceFilterOrg = $("deviceFilterOrg").value || "all";
      renderDevices(state.snapshot?.devices || []);
    });
    document.querySelectorAll(".nav a").forEach((link) => {
      link.addEventListener("click", () => {
        document.querySelectorAll(".nav a").forEach((item) => item.classList.remove("active"));
        link.classList.add("active");
      });
    });

    if (token()) refresh();
  </script>
</body>
</html>
"##;
