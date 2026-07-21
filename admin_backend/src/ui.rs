pub const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>KeyLessPass Admin</title>
  <style>
    :root {
      color-scheme: light;
      --bg: #f5f7f4;
      --panel: #ffffff;
      --panel-2: #eef2ec;
      --text: #18201b;
      --muted: #647064;
      --line: #d9e0d8;
      --accent: #434cff;
      --accent-2: #f4fb59;
      --danger: #d93f3f;
      --ok: #168a4a;
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }
    * { box-sizing: border-box; }
    body { margin: 0; background: var(--bg); color: var(--text); }
    header {
      padding: 24px 32px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      border-bottom: 1px solid var(--line);
      background: rgba(255, 255, 255, 0.86);
      position: sticky;
      top: 0;
      backdrop-filter: blur(12px);
      z-index: 4;
    }
    h1 { margin: 0; font-size: 24px; letter-spacing: 0; }
    h2 { margin: 0 0 14px; font-size: 18px; letter-spacing: 0; }
    h3 { margin: 0 0 8px; font-size: 15px; letter-spacing: 0; }
    p { color: var(--muted); line-height: 1.5; margin: 0 0 12px; }
    main {
      width: min(1320px, calc(100vw - 40px));
      margin: 24px auto 48px;
      display: grid;
      gap: 16px;
    }
    .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; align-items: start; }
    .panel {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 18px;
      box-shadow: 0 16px 40px rgba(22, 28, 22, 0.06);
    }
    .compact { padding: 14px; }
    label { display: block; color: var(--muted); font-size: 12px; font-weight: 700; margin: 12px 0 6px; }
    input, textarea, select {
      width: 100%;
      border: 1px solid var(--line);
      border-radius: 6px;
      background: #fbfcfa;
      color: var(--text);
      padding: 11px 12px;
      font: inherit;
      outline: none;
    }
    textarea { min-height: 130px; resize: vertical; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 12px; }
    input:focus, textarea:focus, select:focus { border-color: var(--accent); box-shadow: 0 0 0 3px rgba(67, 76, 255, 0.12); }
    button {
      border: 0;
      border-radius: 6px;
      padding: 10px 14px;
      background: var(--accent);
      color: white;
      font-weight: 800;
      cursor: pointer;
    }
    button.secondary { background: var(--panel-2); color: var(--text); border: 1px solid var(--line); }
    button.warning { background: var(--danger); }
    button:disabled { opacity: .5; cursor: not-allowed; }
    .row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
    .space { justify-content: space-between; }
    .pill {
      display: inline-flex;
      gap: 6px;
      align-items: center;
      padding: 6px 10px;
      border-radius: 999px;
      background: var(--panel-2);
      color: var(--muted);
      font-size: 12px;
      font-weight: 700;
    }
    .ok { color: var(--ok); }
    .danger { color: var(--danger); }
    table { width: 100%; border-collapse: collapse; font-size: 13px; }
    th, td { text-align: left; padding: 10px 8px; border-bottom: 1px solid var(--line); vertical-align: top; }
    th { color: var(--muted); font-size: 12px; }
    code { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 12px; }
    .mono { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 12px; word-break: break-all; }
    .notice { border-left: 4px solid var(--accent); background: #eef0ff; padding: 12px; border-radius: 6px; }
    .danger-box { border-left: 4px solid var(--danger); background: #fff0f0; padding: 12px; border-radius: 6px; }
    .muted { color: var(--muted); }
    .hidden { display: none; }
    @media (max-width: 900px) {
      header { padding: 18px; align-items: flex-start; gap: 12px; flex-direction: column; }
      main { width: min(100vw - 24px, 1320px); }
      .grid { grid-template-columns: 1fr; }
      table { display: block; overflow-x: auto; white-space: nowrap; }
    }
  </style>
</head>
<body>
  <header>
    <div>
      <h1>KeyLessPass Admin</h1>
      <p>Intranet device authorization and offline license bundle signing.</p>
    </div>
    <div class="row">
      <span class="pill" id="healthPill">Not connected</span>
      <button class="secondary" id="refreshButton">Refresh</button>
    </div>
  </header>

  <main>
    <section class="panel">
      <h2>Admin token</h2>
      <p>Use the token from <code>KEYLESSPASS_ADMIN_TOKEN</code>. The browser keeps it in local storage for this origin.</p>
      <div class="row">
        <input id="tokenInput" type="password" placeholder="Paste admin token">
        <button id="saveTokenButton">Save token</button>
      </div>
    </section>

    <section class="grid">
      <div class="panel">
        <h2>Signing status</h2>
        <p class="notice">Embed this public key in the commercial KeyLessPass client build. The private signing key must stay only on this admin server.</p>
        <div id="statusBox" class="mono muted">No status loaded.</div>
      </div>
      <div class="panel">
        <h2>Create organization</h2>
        <label>Name</label>
        <input id="orgName" placeholder="Acme Railway Group">
        <label>Plan</label>
        <input id="orgPlan" value="enterprise">
        <label>Max seats</label>
        <input id="orgSeats" type="number" min="1" value="25">
        <label>Valid days</label>
        <input id="orgDays" type="number" min="1" value="365">
        <label>Features, comma separated</label>
        <input id="orgFeatures" value="desktop-client">
        <div style="height:12px"></div>
        <button id="createOrgButton">Create organization</button>
      </div>
    </section>

    <section class="grid">
      <div class="panel">
        <h2>Import device request</h2>
        <p>Paste the device authorization request exported from the KeyLessPass desktop app. It contains only commercial device identifiers, not password secrets.</p>
        <label>Organization</label>
        <select id="deviceOrg"></select>
        <label>Seat label</label>
        <input id="deviceSeat" placeholder="Finance laptop 01">
        <label>Request JSON</label>
        <textarea id="deviceRequestJson" placeholder='{"schemaVersion":1,...}'></textarea>
        <button id="importDeviceButton">Import request</button>
      </div>
      <div class="panel">
        <h2>Issue license bundle</h2>
        <p>Select an organization and devices, then issue a signed bundle that the desktop client can import offline.</p>
        <label>Organization</label>
        <select id="issueOrg"></select>
        <label>Valid days override</label>
        <input id="issueDays" type="number" min="1" placeholder="Leave empty to use organization expiry">
        <div class="row" style="margin: 12px 0;">
          <button id="selectOrgDevicesButton" class="secondary">Select all organization devices</button>
          <button id="issueBundleButton">Issue signed bundle</button>
        </div>
        <textarea id="bundleOutput" readonly placeholder="Signed bundle JSON appears here."></textarea>
        <div class="row">
          <button class="secondary" id="copyBundleButton">Copy bundle</button>
          <button class="secondary" id="downloadBundleButton">Download .klp-license-bundle</button>
        </div>
      </div>
    </section>

    <section class="panel">
      <div class="row space">
        <h2>Devices</h2>
        <span class="pill" id="deviceCountPill">0 devices</span>
      </div>
      <table>
        <thead>
          <tr>
            <th></th>
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
    </section>

    <section class="grid">
      <div class="panel">
        <h2>Issued bundles</h2>
        <table>
          <thead><tr><th>Bundle</th><th>Devices</th><th>Issued</th><th></th></tr></thead>
          <tbody id="bundlesTable"></tbody>
        </table>
      </div>
      <div class="panel">
        <h2>Grants</h2>
        <table>
          <thead><tr><th>Grant</th><th>Seat</th><th>Status</th><th></th></tr></thead>
          <tbody id="grantsTable"></tbody>
        </table>
      </div>
    </section>

    <section class="panel danger-box">
      <h2>Security boundary</h2>
      <p>This admin backend signs commercial authorization metadata only. It must not receive mnemonic phrases, Kmaster, deviceSecret, usbSecret, service passwords, derived passwords, CDR secrets, or recovery wrapper keys.</p>
    </section>
  </main>

  <script>
    const state = { snapshot: null, selectedDeviceIds: new Set(), lastBundle: "" };
    const $ = (id) => document.getElementById(id);

    function token() { return localStorage.getItem("klpAdminToken") || ""; }
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
    function setHealth(text, ok) {
      $("healthPill").textContent = text;
      $("healthPill").className = `pill ${ok ? "ok" : "danger"}`;
    }
    function featuresFromInput(value) {
      return value.split(",").map((v) => v.trim()).filter(Boolean);
    }
    async function refresh() {
      try {
        state.snapshot = await api("/api/snapshot");
        render();
        setHealth("Connected", true);
      } catch (err) {
        setHealth(err.message, false);
      }
    }
    function render() {
      const snap = state.snapshot;
      if (!snap) return;
      $("statusBox").innerHTML = [
        `service: ${snap.status.service}`,
        `keyId: ${snap.status.keyId}`,
        `publicKeyB64: ${snap.status.publicKeyB64}`,
        `publicKeyB64Url: ${snap.status.publicKeyB64url}`,
        `database: ${snap.status.databasePath}`,
        `organizations: ${snap.status.organizationCount}`,
        `devices: ${snap.status.deviceCount}`,
        `bundles: ${snap.status.bundleCount}`,
      ].map(escapeHtml).join("<br>");
      renderOrgSelects(snap.organizations);
      renderDevices(snap.devices);
      renderBundles(snap.bundles);
      renderGrants(snap.grants);
    }
    function renderOrgSelects(orgs) {
      for (const id of ["deviceOrg", "issueOrg"]) {
        const selected = $(id).value;
        $(id).innerHTML = orgs.map((org) => `<option value="${escapeAttr(org.id)}">${escapeHtml(org.name)} (${escapeHtml(org.id)})</option>`).join("");
        if (selected) $(id).value = selected;
      }
    }
    function renderDevices(devices) {
      $("deviceCountPill").textContent = `${devices.length} devices`;
      $("devicesTable").innerHTML = devices.map((device) => `
        <tr>
          <td><input type="checkbox" data-device-id="${escapeAttr(device.id)}" ${state.selectedDeviceIds.has(device.id) ? "checked" : ""}></td>
          <td>${escapeHtml(device.seatLabel)}</td>
          <td><code>${escapeHtml(device.organizationId)}</code></td>
          <td class="mono">${escapeHtml(device.commercialDeviceId)}</td>
          <td class="mono">${escapeHtml(device.deviceFingerprint)}</td>
          <td>${escapeHtml(device.platform)} ${escapeHtml(device.appVersion)}</td>
          <td>${escapeHtml(device.updatedAt)}</td>
        </tr>
      `).join("");
      $("devicesTable").querySelectorAll("input[type=checkbox]").forEach((box) => {
        box.addEventListener("change", () => {
          if (box.checked) state.selectedDeviceIds.add(box.dataset.deviceId);
          else state.selectedDeviceIds.delete(box.dataset.deviceId);
        });
      });
    }
    function renderBundles(bundles) {
      $("bundlesTable").innerHTML = bundles.map((bundle) => `
        <tr>
          <td class="mono">${escapeHtml(bundle.bundleId)}</td>
          <td>${bundle.deviceCount}</td>
          <td>${escapeHtml(bundle.issuedAt)}</td>
          <td><button class="secondary" data-bundle-id="${escapeAttr(bundle.bundleId)}">Load</button></td>
        </tr>
      `).join("");
      $("bundlesTable").querySelectorAll("button").forEach((button) => {
        button.addEventListener("click", () => {
          const bundle = state.snapshot.bundles.find((item) => item.bundleId === button.dataset.bundleId);
          if (bundle) setBundleOutput(bundle.envelopeJson);
        });
      });
    }
    function renderGrants(grants) {
      $("grantsTable").innerHTML = grants.map((grant) => `
        <tr>
          <td class="mono">${escapeHtml(grant.grantId)}</td>
          <td>${escapeHtml(grant.seatLabel)}</td>
          <td>${grant.revoked ? '<span class="danger">revoked</span>' : '<span class="ok">active</span>'}</td>
          <td>${grant.revoked ? "" : `<button class="warning" data-grant-id="${escapeAttr(grant.grantId)}">Revoke</button>`}</td>
        </tr>
      `).join("");
      $("grantsTable").querySelectorAll("button").forEach((button) => {
        button.addEventListener("click", async () => {
          if (!confirm("Revoke this device grant? Issue a new bundle after revocation so clients can import the revocation list.")) return;
          await api(`/api/grants/${encodeURIComponent(button.dataset.grantId)}/revoke`, { method: "POST", body: "{}" });
          await refresh();
        });
      });
    }
    function setBundleOutput(value) {
      state.lastBundle = value;
      $("bundleOutput").value = value;
    }
    function escapeHtml(value) {
      return String(value ?? "").replace(/[&<>"']/g, (ch) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[ch]));
    }
    function escapeAttr(value) { return escapeHtml(value).replace(/`/g, "&#96;"); }

    $("tokenInput").value = token();
    $("saveTokenButton").addEventListener("click", async () => {
      localStorage.setItem("klpAdminToken", $("tokenInput").value.trim());
      await refresh();
    });
    $("refreshButton").addEventListener("click", refresh);
    $("createOrgButton").addEventListener("click", async () => {
      await api("/api/organizations", {
        method: "POST",
        body: JSON.stringify({
          name: $("orgName").value,
          plan: $("orgPlan").value,
          maxSeats: Number($("orgSeats").value || 25),
          validDays: Number($("orgDays").value || 365),
          features: featuresFromInput($("orgFeatures").value),
        }),
      });
      $("orgName").value = "";
      await refresh();
    });
    $("importDeviceButton").addEventListener("click", async () => {
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
    });
    $("selectOrgDevicesButton").addEventListener("click", () => {
      const orgId = $("issueOrg").value;
      state.selectedDeviceIds = new Set((state.snapshot?.devices || []).filter((device) => device.organizationId === orgId).map((device) => device.id));
      renderDevices(state.snapshot?.devices || []);
    });
    $("issueBundleButton").addEventListener("click", async () => {
      const payload = {
        organizationId: $("issueOrg").value,
        deviceIds: Array.from(state.selectedDeviceIds),
      };
      if ($("issueDays").value) payload.validDays = Number($("issueDays").value);
      const issued = await api("/api/licenses/issue", { method: "POST", body: JSON.stringify(payload) });
      setBundleOutput(issued.envelopeJson);
      await refresh();
    });
    $("copyBundleButton").addEventListener("click", async () => {
      await navigator.clipboard.writeText($("bundleOutput").value);
    });
    $("downloadBundleButton").addEventListener("click", () => {
      const blob = new Blob([$("bundleOutput").value], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = "keylesspass.klp-license-bundle";
      link.click();
      URL.revokeObjectURL(url);
    });
    if (token()) refresh();
  </script>
</body>
</html>
"#;
