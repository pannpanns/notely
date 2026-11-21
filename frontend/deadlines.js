/* Notely Deadline Notes: Calendar view, list view, consumer-friendly Google Calendar integration, and AI extraction from text & screenshots. */
'use strict';

state.deadlines = state.deadlines || [];
state.deadlineView = state.deadlineView || 'calendar';
state.deadlineFilter = state.deadlineFilter || 'all';
state.calendarYear = state.calendarYear || new Date().getFullYear();
state.calendarMonth = state.calendarMonth ?? new Date().getMonth();
state.selectedDeadlineDate = state.selectedDeadlineDate || null;
state.deadlineSearch = state.deadlineSearch || '';
state.deadlineAiPrompt = state.deadlineAiPrompt || '';
state.deadlineAiImage = state.deadlineAiImage || null;
state.deadlineAiLoading = false;
state.deadlineAiStatus = '';
state.extractedPreview = [];
state.deadlineTab = state.deadlineTab || 'prompt'; // 'prompt' | 'screenshot'

// Google Calendar Integration State (Privacy First & Consumer Friendly)
state.gcalStatus = state.gcalStatus || {
  connected: false,
  email: '',
  autoSync: true,
  hasCredentials: false,
  clientId: '',
  dismissPrompt: false
};
state.gcalConnecting = false;
state.gcalMenuOpen = false;
state.gcalPromptShownSession = false;
state.showGcalEmail = false; // Mask email by default to protect user privacy
window.__gcalPollTimer = null;

// Helper: Mask email for privacy protection (e.g. evanjagadish@gmail.com -> ev••••••h@gmail.com)
function maskEmail(email) {
  if (!email || typeof email !== 'string' || !email.includes('@')) return 'Akun Google Terhubung';
  const parts = email.split('@');
  const name = parts[0];
  const domain = parts[1];
  if (name.length <= 2) return `${name[0]}•@${domain}`;
  const start = name.slice(0, 2);
  const end = name.slice(-1);
  const hidden = '•'.repeat(Math.min(6, Math.max(3, name.length - 3)));
  return `${start}${hidden}${end}@${domain}`;
}

// --------------------------------------------------------------------------
// GOOGLE CALENDAR API & FLOW
// --------------------------------------------------------------------------

// Standing automatic two-way background polling for Google Calendar (pull smartphone / web changes every 25s)
function startGcalBackgroundPolling() {
  if (window.__gcalBackgroundPollInterval) return;
  window.__gcalBackgroundPollInterval = setInterval(async () => {
    try {
      if (typeof api !== 'function' || !engine.ready) return;
      if (!state.gcalStatus || !state.gcalStatus.connected) return;

      const res = await api('/gcalendar/pull', {});
      if (res && (res.imported > 0 || res.updated > 0)) {
        await loadDeadlinesData();
        if (state.page === 'deadlines') {
          refreshPage();
        }
      }
    } catch (e) {
      // Passive background polling must remain silent
    }
  }, 25000);
}

async function loadGcalStatus() {
  try {
    if (typeof api === 'function' && engine.ready) {
      const data = await api('/gcalendar/status');
      if (data && typeof data.connected === 'boolean') {
        state.gcalStatus = data;
        safeStore.set('notelyGcalStatus', data);
        if (data.connected) {
          startGcalBackgroundPolling();
          if (!window.__lastGcalPullTime || Date.now() - window.__lastGcalPullTime > 15000) {
            window.__lastGcalPullTime = Date.now();
            api('/gcalendar/pull', {}).then(res => {
              if (res && (res.imported > 0 || res.updated > 0)) {
                loadDeadlinesData().then(() => {
                  if (state.page === 'deadlines') refreshPage();
                });
              }
            }).catch(() => {});
          }
        }
      }
    } else {
      const local = safeStore.get('notelyGcalStatus');
      if (local) state.gcalStatus = local;
    }
  } catch (e) {
    const local = safeStore.get('notelyGcalStatus');
    if (local) state.gcalStatus = local;
  }
}

// Automatically check and open connect popup when visiting deadlines page (if not connected & not dismissed)
function checkGcalPromptOnEnter() {
  if (state.gcalPromptShownSession) return;
  state.gcalPromptShownSession = true;

  setTimeout(() => {
    if (!state.gcalStatus.connected && !state.gcalStatus.dismissPrompt) {
      openGcalConnectModal(false);
    }
  }, 450);
}

function openGcalConnectModal(isUserAction = true) {
  state.gcalConnecting = false;
  renderGcalModal();
}

function closeGcalModal() {
  if (window.__gcalPollTimer) {
    clearInterval(window.__gcalPollTimer);
    window.__gcalPollTimer = null;
  }
  state.gcalConnecting = false;
  const el = document.getElementById('gcal-connect-modal');
  if (el) el.remove();
}

function openGcalCredsModal() {
  renderGcalCredsModal();
}

function closeGcalCredsModal() {
  const el = document.getElementById('gcal-creds-modal');
  if (el) el.remove();
}

function openGcalUserGuideModal() {
  renderGcalUserGuideModal();
}

function closeGcalUserGuideModal() {
  const el = document.getElementById('gcal-user-guide-modal');
  if (el) el.remove();
}

function renderGcalUserGuideModal() {
  let modalEl = document.getElementById('gcal-user-guide-modal');
  if (!modalEl) {
    modalEl = document.createElement('div');
    modalEl.id = 'gcal-user-guide-modal';
    modalEl.className = 'overlay';
    document.body.appendChild(modalEl);
  }

  modalEl.innerHTML = `<div class="gcal-guide-modal-card" onclick="event.stopPropagation()">
    <div class="space-between" style="align-items:flex-start;margin-bottom:14px">
      <div style="display:flex;align-items:center;gap:12px">
        <div class="gcal-icon-badge" style="background:#edf7f2;color:#18786c">${icon('google')}</div>
        <div>
          <h2 style="font-size:18px;font-weight:700;color:#183226;margin:0 0 2px;line-height:1.25">Panduan Mudah Otorisasi Google Calendar</h2>
          <span style="font-size:12.5px;color:#677d71">Cara menghubungkan Notely ke Google Calendar tanpa pusing</span>
        </div>
      </div>
      <button class="dismiss" data-action="close-gcal-guide-modal" title="Tutup">×</button>
    </div>

    <p style="font-size:13px;color:#576f62;line-height:1.6;margin:0 0 16px">
      Google memiliki standar keamanan tinggi untuk melindungi akun Anda. Saat menghubungkan Notely pertama kali, ikuti 4 langkah sederhana berikut:
    </p>

    <div class="gcal-guide-steps">
      <div class="gcal-guide-step-card">
        <div class="gcal-step-num">1</div>
        <div class="gcal-step-content">
          <h5>1. Klik tombol "Sambungkan Akun Google"</h5>
          <p>Browser bawaan komputer Anda akan otomatis terbuka ke situs resmi Google (<code>accounts.google.com</code>).</p>
        </div>
      </div>

      <div class="gcal-guide-step-card">
        <div class="gcal-step-num">2</div>
        <div class="gcal-step-content">
          <h5>2. Pilih Akun Google Anda</h5>
          <p>Pilih akun Gmail atau akun kampus yang biasa Anda pakai untuk melihat kalender di HP atau laptop.</p>
        </div>
      </div>

      <div class="gcal-guide-step-card">
        <div class="gcal-step-num">3</div>
        <div class="gcal-step-content">
          <h5>3. Jika Muncul Pesan "Google belum memverifikasi aplikasi ini"</h5>
          <p><strong>Jangan khawatir, ini sangat normal!</strong> Notely berjalan mandiri di laptop pribadi Anda secara offline (bukan aplikasi cloud komersial publik).</p>
          <div class="gcal-tip-box">
            👉 Cukup klik tulisan kecil <strong>"Lanjutan" (Advanced)</strong> di bagian bawah layar, lalu klik <strong>"Buka Notely (tidak aman)" / "Go to Notely"</strong>.
          </div>
        </div>
      </div>

      <div class="gcal-guide-step-card">
        <div class="gcal-step-num">4</div>
        <div class="gcal-step-content">
          <h5>4. Beri Izin &amp; Klik "Lanjutkan" / "Izinkan"</h5>
          <p>Beri tanda centang pada izin kalender, lalu klik <strong>Lanjutkan</strong>. Jendela peramban akan otomatis menutup dan Notely langsung terhubung seketika!</p>
        </div>
      </div>
    </div>

    <div style="background:#f4faf6;border:1px solid #d5e9dc;border-radius:12px;padding:16px 18px;margin-bottom:20px;display:flex;align-items:center;justify-content:space-between;gap:14px;flex-wrap:wrap">
      <div>
        <strong style="display:block;font-size:13px;color:#184c31;margin-bottom:3px">Tidak ingin repot dengan izin Google?</strong>
        <p style="font-size:12px;color:#5a7867;margin:0">Gunakan Mode Cepat Web. Jadwal tugas akan otomatis membuka formulir Google Calendar yang sudah terisi lengkap tanpa setup akun apapun!</p>
      </div>
      <button class="button accent small" data-action="choose-gcal-web-mode">
        ${icon('globe')} Pakai Mode Cepat Saja
      </button>
    </div>

    <div style="display:flex;justify-content:space-between;align-items:center;padding-top:14px;border-top:1px solid #e8efe9;flex-wrap:wrap;gap:10px">
      <button class="button ghost" data-action="open-gcal-creds-modal" style="font-size:11.5px;color:#677d71">
        ${icon('sliders')} Kredensial Khusus Developer (Opsional)
      </button>
      <div style="display:flex;gap:8px">
        <button class="button ghost" data-action="close-gcal-guide-modal">Kembali</button>
        <button class="button accent" data-action="start-gcal-auth-flow">
          ${icon('google')} Sambungkan Sekarang
        </button>
      </div>
    </div>
  </div>`;
}

function renderGcalCredsModal() {
  let modalEl = document.getElementById('gcal-creds-modal');
  if (!modalEl) {
    modalEl = document.createElement('div');
    modalEl.id = 'gcal-creds-modal';
    modalEl.className = 'overlay';
    document.body.appendChild(modalEl);
  }

  const clientId = state.gcalStatus.clientId || '';
  const hasSecret = !!state.gcalStatus.hasCredentials;
  if (!state.gcalCredsTab) {
    state.gcalCredsTab = clientId ? 'manual' : 'json';
  }

  modalEl.innerHTML = `<div class="gcal-creds-modal-card" onclick="event.stopPropagation()">
    <!-- Header -->
    <div class="gcal-modal-header">
      <div style="display:flex;align-items:center;gap:12px">
        <div class="gcal-icon-badge">${icon('google')}</div>
        <div>
          <h2 style="font-size:18px;font-weight:700;color:#183226;margin:0 0 2px">Konfigurasi Google Cloud (Developer)</h2>
          <span style="font-size:12px;color:#677d71">Gunakan OAuth Client ID pribadi untuk sinkronisasi otomatis</span>
        </div>
      </div>
      <button class="dismiss" data-action="close-gcal-creds-modal" title="Tutup">×</button>
    </div>

    <!-- Scrollable Body -->
    <div class="gcal-modal-body">
      <!-- Slim Callout Banner for Fast Web Mode -->
      <div class="gcal-slim-banner">
        <div style="display:flex;align-items:center;gap:8px">
          <span style="display:inline-flex;color:#18786c">${icon('spark')}</span>
          <span style="font-size:12px;color:#2a503e">
            Bukan developer? Anda cukup pakai <strong>Mode Cepat Web</strong> (1-klik tanpa konfigurasi).
          </span>
        </div>
        <button type="button" class="button small ghost" data-action="choose-gcal-web-mode" style="color:#18786c;font-weight:600;font-size:11.5px;padding:3px 10px;height:28px">
          Pakai Mode Cepat
        </button>
      </div>

      <!-- Segmented Tab Picker: File JSON vs Masukkan Manual -->
      <div class="gcal-tabs-container">
        <button type="button" class="gcal-tab-btn ${state.gcalCredsTab !== 'manual' ? 'active' : ''}" data-action="switch-creds-tab" data-id="json">
          ${icon('folder')} Unggah File JSON (Rekomendasi)
        </button>
        <button type="button" class="gcal-tab-btn ${state.gcalCredsTab === 'manual' ? 'active' : ''}" data-action="switch-creds-tab" data-id="manual">
          ${icon('sliders')} Masukkan Manual
        </button>
      </div>

      <!-- Tab Content -->
      ${state.gcalCredsTab !== 'manual' ? `
        <!-- Tab 1: Dropzone JSON -->
        <div class="gcal-drop-zone" id="gcal-creds-dropzone" onclick="document.getElementById('gcal-json-file-input').click()">
          ${icon('folder')}
          <strong>Tarik &amp; lepas file <code>client_secret_*.json</code> ke sini</strong>
          <p>atau klik untuk memilih file hasil download dari Google Cloud Console</p>
          ${clientId ? `
            <div class="gcal-loaded-pill">
              ✓ Client ID terpasang: <strong>${esc(clientId.slice(0, 30))}...</strong>
            </div>
          ` : ''}
        </div>
        <input type="file" id="gcal-json-file-input" accept=".json" hidden style="display:none !important;" onchange="window.handleGcalJsonFileSelect(this)">
      ` : `
        <!-- Tab 2: Manual Inputs -->
        <div class="gcal-manual-form">
          <div class="field" style="margin:0">
            <label class="field-label" for="gcal-input-client-id" style="font-size:12.5px;font-weight:600;color:#1e352a;margin-bottom:4px">OAuth Client ID (Desktop app) *</label>
            <input type="text" id="gcal-input-client-id" class="textbox" style="font-size:12.5px;height:38px;border-radius:8px" placeholder="Contoh: 123456789-abcdef.apps.googleusercontent.com" value="${esc(clientId)}" autocomplete="off" spellcheck="false">
          </div>
          <div class="field" style="margin:0">
            <label class="field-label" for="gcal-input-client-secret" style="font-size:12.5px;font-weight:600;color:#1e352a;margin-bottom:4px">Client Secret *</label>
            <input type="password" id="gcal-input-client-secret" class="textbox" style="font-size:12.5px;height:38px;border-radius:8px" placeholder="${hasSecret ? '••••••••••••••••••••••••••••••••' : 'Contoh: GOCSPX-xxxxxxxxxxxxxxxxxxxxxxxx'}" autocomplete="off" spellcheck="false">
            <span class="field-help" style="font-size:11px;color:#74897d;margin-top:3px">Tersimpan aman secara lokal di komputer Anda.</span>
          </div>
        </div>
      `}

      <!-- Panduan Singkat Card -->
      <div class="gcal-guide-card">
        <div class="gcal-guide-head">
          <strong style="font-size:12.5px;color:#1c392b">Panduan Singkat Google Cloud:</strong>
          <button type="button" class="button small ghost gcal-console-btn" data-action="open-gcal-console-link" title="Buka Google Cloud Console">
            ${icon('external')} Buka Console Credentials ↗
          </button>
        </div>
        <ol class="gcal-guide-list">
          <li>Buat <strong>Project Baru</strong> di Google Cloud Console.</li>
          <li>Aktifkan <strong>Google Calendar API</strong> di menu <em>Enabled APIs &amp; Services &gt; Library</em>.</li>
          <li>Pada menu <strong>Audience (Consent Screen)</strong>: Pilih <strong>External</strong>, lalu tambahkan email akun Google Anda pada <strong>Test users</strong> (Wajib, agar tidak Error 403).</li>
          <li>Pada menu <strong>Clients / Credentials</strong>: Klik <strong>+ Create Credentials &gt; OAuth client ID</strong>, pilih tipe <strong>Desktop app</strong>, lalu klik Create.</li>
          <li>Unduh JSON lalu seret ke tab <em>Unggah File JSON</em>, atau salin Client ID &amp; Secret ke tab <em>Masukkan Manual</em>.</li>
        </ol>
      </div>
    </div>

    <!-- Fixed Footer (Always visible!) -->
    <div class="gcal-modal-footer">
      <div style="font-size:11.5px;color:#71877a">
        ${hasSecret ? '✓ Kredensial aktif tersimpan' : 'Wajib diisi untuk OAuth'}
      </div>
      <div style="display:flex;gap:10px;align-items:center">
        <button class="button ghost" data-action="close-gcal-creds-modal" style="height:36px;padding:0 14px">Batal</button>
        <button class="button accent" data-action="save-gcal-creds-and-auth" style="height:36px;padding:0 18px;font-weight:600">
          ${icon('google')} Simpan &amp; Lanjutkan Login
        </button>
      </div>
    </div>
  </div>`;

  setupGcalDropzone();
}

window.handleGcalJsonFileSelect = function(input) {
  const file = input?.files?.[0];
  if (!file) return;
  parseAndApplyGcalJsonFile(file);
};

function parseAndApplyGcalJsonFile(file) {
  const reader = new FileReader();
  reader.onload = async (e) => {
    try {
      const text = e.target.result;
      const json = JSON.parse(text);
      const creds = json.installed || json.web || json;
      const cid = creds.client_id;
      const csec = creds.client_secret;
      if (!cid) {
        throw Error("File JSON tidak memuat 'client_id' yang valid.");
      }
      state.gcalStatus.clientId = cid;
      if (csec) state.gcalStatus.clientSecret = csec;
      state.gcalStatus.hasCredentials = true;

      const idInput = document.getElementById('gcal-input-client-id');
      const secInput = document.getElementById('gcal-input-client-secret');
      if (idInput) idInput.value = cid;
      if (secInput && csec) secInput.value = csec;

      renderGcalCredsModal();
      toast('✓ File kredensial Google berhasil dimuat!');
    } catch (err) {
      alertError(err);
    }
  };
  reader.readAsText(file);
}

function setupGcalDropzone() {
  const zone = document.getElementById('gcal-creds-dropzone');
  if (!zone) return;
  zone.ondragover = (e) => { e.preventDefault(); zone.classList.add('dragover'); };
  zone.ondragleave = () => { zone.classList.remove('dragover'); };
  zone.ondrop = (e) => {
    e.preventDefault();
    zone.classList.remove('dragover');
    const file = e.dataTransfer?.files?.[0];
    if (file) parseAndApplyGcalJsonFile(file);
  };
}

async function saveGcalCredsAndAuth() {
  let cid = document.getElementById('gcal-input-client-id')?.value?.trim();
  let csec = document.getElementById('gcal-input-client-secret')?.value?.trim();
  if (!cid) cid = state.gcalStatus.clientId;
  if (!csec) csec = state.gcalStatus.clientSecret;

  if (!cid) {
    alertError(new Error("Masukkan Google OAuth Client ID atau unggah file JSON terlebih dahulu."));
    return;
  }
  try {
    const payload = { clientId: cid };
    if (csec) payload.clientSecret = csec;
    await updateGcalConfig(payload);
    state.gcalStatus.clientId = cid;
    if (csec) state.gcalStatus.hasCredentials = true;
    closeGcalCredsModal();
    toast('Kredensial disimpan. Membuka login Google…');
    startGcalAuth();
  } catch (err) {
    alertError(err);
  }
}

function renderGcalModal() {
  let modalEl = document.getElementById('gcal-connect-modal');
  if (!modalEl) {
    modalEl = document.createElement('div');
    modalEl.id = 'gcal-connect-modal';
    modalEl.className = 'overlay';
    document.body.appendChild(modalEl);
  }

  const isConnected = state.gcalStatus.connected;
  const isWebMode = state.gcalStatus.preferredMode === 'web';

  modalEl.innerHTML = `<div class="gcal-modal-card gcal-options-modal" onclick="event.stopPropagation()">
    <div class="space-between" style="align-items:flex-start;margin-bottom:12px">
      <div style="display:flex;align-items:center;gap:12px">
        <div class="gcal-logo-circle" style="margin-bottom:0;width:44px;height:44px">${icon('google')}</div>
        <div>
          <h2 class="gcal-modal-title" style="margin:0 0 2px;line-height:1.25">Integrasi Google Calendar</h2>
          <span style="font-size:12.5px;color:#677d70">Pilih solusi sinkronisasi tenggat tugas yang paling cocok untuk Anda:</span>
        </div>
      </div>
      <button class="dismiss" data-action="close-gcal-modal" title="Tutup">×</button>
    </div>

    ${isConnected ? `
      <!-- Connected State -->
      <div class="gcal-success-box" style="margin-top:14px">
        <div class="gcal-success-icon">✓</div>
        <strong>Google Calendar Terhubung Resmi</strong>
        <p>Akun <strong>${esc(state.gcalStatus.email || 'Google')}</strong> aktif tersambung via OAuth 2.0. Seluruh tugas otomatis tersinkron ke kalender ponsel &amp; laptop.</p>
      </div>

      <div class="gcal-modal-footer" style="justify-content:flex-end">
        <button class="button accent" data-action="close-gcal-modal">Selesai</button>
      </div>
    ` : state.gcalConnecting ? `
      <!-- Connecting State -->
      <div class="gcal-waiting-box" style="margin-top:14px">
        <div class="gcal-spinner"></div>
        <strong>Menunggu Otorisasi di Peramban…</strong>
        <p>Laman login Google telah dibuka di browser Anda. Pilih akun Anda lalu klik <strong>"Izinkan" / "Allow"</strong>.</p>
        <div style="font-size:11.5px;color:#788e81;margin-top:4px">Jika muncul "Google belum memverifikasi aplikasi", klik <em>Lanjutan</em> lalu <em>Buka Notely</em>.</div>
      </div>

      <div class="gcal-modal-footer" style="justify-content:flex-end">
        <button class="button danger" data-action="cancel-gcal-auth">Batal</button>
      </div>
    ` : `
      <!-- Initial Connect: Clean & Spacious Choices -->
      <div class="gcal-choices-grid">
        <!-- Pilihan 1: Mode Cepat Web (Paling Praktis) -->
        <div class="gcal-choice-card ${isWebMode ? 'featured' : ''}">
          <div class="gcal-choice-badge recommended">Mode Cepat</div>
          <div class="gcal-choice-header">
            <div class="gcal-choice-icon">${icon('globe')}</div>
            <div>
              <h4>Mode Cepat Web</h4>
              <span class="subtext">1-Klik Tanpa Perlu Setup Akun</span>
            </div>
          </div>
          <p class="gcal-choice-desc">
            Saat menyimpan tugas, formulir Google Calendar otomatis terbuka di peramban dengan judul, tanggal, dan jam yang sudah terisi. Anda tinggal menekan tombol <strong>Simpan</strong> satu kali.
          </p>
          <div style="margin-top:auto;padding-top:10px">
            <button class="button accent" style="width:100%" data-action="choose-gcal-web-mode">
              ${icon('globe')} Gunakan Mode Cepat
            </button>
          </div>
        </div>

        <!-- Pilihan 2: Kredensial Khusus Developer (OAuth API) -->
        <div class="gcal-choice-card">
          <div class="gcal-choice-badge">Pengembang / Lanjutan</div>
          <div class="gcal-choice-header">
            <div class="gcal-choice-icon accent-bg">${icon('sliders')}</div>
            <div>
              <h4>Kredensial Khusus</h4>
              <span class="subtext">Google Calendar API Pribadi</span>
            </div>
          </div>
          <p class="gcal-choice-desc">
            Sambungkan akun Google Anda dengan Client ID atau file JSON dari Google Cloud Console. Tugas akan otomatis tersinkronisasi di latar belakang tanpa perlu membuka peramban.
          </p>
          <div style="margin-top:auto;padding-top:10px">
            <button class="button neutral" style="width:100%" data-action="open-gcal-creds-modal">
              ${icon('settings')} Atur Kredensial Pribadi
            </button>
          </div>
        </div>
      </div>

      <div class="gcal-modal-footer" style="margin-top:14px;padding-top:12px;border-top:1px solid #e7eee9">
        <label class="gcal-dismiss-check">
          <input type="checkbox" id="gcal-dont-show-again">
          <span>Jangan ingatkan lagi saat membuka halaman</span>
        </label>
        <button class="button ghost" data-action="dismiss-gcal-modal">Tutup</button>
      </div>
    `}
  </div>`;
}

function openExternalUrl(url) {
  if (!url) return;
  // 1. Try Tauri native explorer invoke first
  try {
    if (window.__TAURI__?.core?.invoke) {
      window.__TAURI__.core.invoke('open_browser_url', { url }).catch(() => {
        if (typeof api === 'function' && engine.ready) {
          api('/open-url', { url }).catch(() => {});
        }
      });
      return;
    }
  } catch (e) {}

  // 2. Python backend /open-url
  if (typeof api === 'function' && engine.ready) {
    api('/open-url', { url }).catch(() => {});
  }

  // 3. Browser fallback
  try {
    window.open(url, '_blank');
  } catch (e) {}
}
window.openExternalUrl = openExternalUrl;

// Start Google Calendar authentication via authentic Google OAuth 2.0
async function startGcalAuth() {
  if (!state.gcalStatus.hasCredentials && !state.gcalStatus.clientId) {
    openGcalCredsModal();
    return;
  }

  state.gcalConnecting = true;
  renderGcalModal();

  try {
    if (typeof api === 'function' && engine.ready) {
      const res = await api('/gcalendar/auth-url', {});
      if (!res.authUrl) {
        throw Error(res.message || 'Gagal memulai otorisasi Google Calendar.');
      }

      openExternalUrl(res.authUrl);

      // Poll until local loopback receives Google OAuth callback
      let attempts = 0;
      if (window.__gcalPollTimer) clearInterval(window.__gcalPollTimer);
      window.__gcalPollTimer = setInterval(async () => {
        attempts++;
        if (attempts > 90 || !state.gcalConnecting) {
          clearInterval(window.__gcalPollTimer);
          window.__gcalPollTimer = null;
          if (state.gcalConnecting) {
            state.gcalConnecting = false;
            renderGcalModal();
          }
          return;
        }
        try {
          const st = await api('/gcalendar/status');
          if (st && st.connected && (st.mode === 'oauth' || st.email)) {
            clearInterval(window.__gcalPollTimer);
            window.__gcalPollTimer = null;
            state.gcalConnecting = false;
            state.gcalStatus = st;
            safeStore.set('notelyGcalStatus', state.gcalStatus);
            startGcalBackgroundPolling();
            api('/gcalendar/pull', {}).then(res => {
              if (res && (res.imported > 0 || res.updated > 0)) {
                loadDeadlinesData().then(() => refreshPage());
              }
            }).catch(() => {});
            closeGcalModal();
            refreshPage();
            toast(`✓ Akun Google (${st.email || 'OAuth'}) resmi tersambung!`);
          }
        } catch (e) {}
      }, 1500);
      return;
    } else {
      // Prototype simulation fallback
      await new Promise(r => setTimeout(r, 800));
      state.gcalStatus = {
        connected: true,
        mode: 'oauth',
        email: 'user.mahasiswa@gmail.com',
        autoSync: true,
        hasCredentials: true,
        dismissPrompt: false
      };
      state.gcalConnecting = false;
      safeStore.set('notelyGcalStatus', state.gcalStatus);
      renderGcalModal();
      refreshPage();
      toast('✓ Akun Google resmi tersambung (Mode Simulasi)!');
    }
  } catch (err) {
    state.gcalConnecting = false;
    renderGcalModal();
    alertError(err);
  }
}

// Disconnect Google Calendar
async function disconnectGcal() {
  try {
    if (typeof api === 'function' && engine.ready) {
      await api('/gcalendar/disconnect', {});
    }
    state.gcalStatus.connected = false;
    state.gcalStatus.email = '';
    state.gcalStatus.mode = 'direct';
    state.gcalMenuOpen = false;
    safeStore.set('notelyGcalStatus', state.gcalStatus);
    toast('Google Calendar berhasil diputuskan.');
    refreshPage();
  } catch (err) {
    alertError(err);
  }
}

// Update Google Calendar config
async function updateGcalConfig(updates) {
  try {
    if (typeof api === 'function' && engine.ready) {
      const res = await api('/gcalendar/config', updates);
      if (res.config) {
        state.gcalStatus = { ...state.gcalStatus, ...res.config };
      }
    } else {
      state.gcalStatus = { ...state.gcalStatus, ...updates };
    }
    safeStore.set('notelyGcalStatus', state.gcalStatus);
    refreshPage();
  } catch (err) {
    alertError(err);
  }
}

// Helper to build direct Google Calendar event creation URL template
function buildGcalWebUrl(item) {
  const title = item.title || 'Tenggat Tugas';
  const course = item.course || '';
  const summary = course ? `${title} [${course}]` : title;
  const desc = `${item.description || ''}\n\nMata Kuliah / Kategori: ${course || '-'}\nPrioritas: ${(item.priority || 'sedang').toUpperCase()}\n\n[Dibuat & Dikelola via Notely Desktop]`.trim();

  const deadlineDate = item.deadline || new Date().toISOString().slice(0, 10);
  const timeStr = item.time || '23:59';
  const cleanDate = deadlineDate.replace(/-/g, '');

  let datesParam;
  if (timeStr) {
    const parts = timeStr.split(':');
    const h = parseInt(parts[0], 10) || 0;
    const m = parseInt(parts[1], 10) || 0;
    const totalMin = h * 60 + m;
    const startMin = Math.max(0, totalMin - 30);
    const startH = Math.floor(startMin / 60);
    const startM = startMin % 60;
    const cleanStart = String(startH).padStart(2, '0') + String(startM).padStart(2, '0') + '00';
    const cleanEnd = String(h).padStart(2, '0') + String(m).padStart(2, '0') + '00';
    datesParam = `${cleanDate}T${cleanStart}/${cleanDate}T${cleanEnd}`;
  } else {
    const dt = new Date(deadlineDate + 'T00:00:00');
    dt.setDate(dt.getDate() + 1);
    const nextCleanDate = `${dt.getFullYear()}${String(dt.getMonth() + 1).padStart(2, '0')}${String(dt.getDate()).padStart(2, '0')}`;
    datesParam = `${cleanDate}/${nextCleanDate}`;
  }

  // Detect user's local timezone (e.g. Asia/Jakarta, Asia/Makassar, Asia/Shanghai, etc.)
  let userTz = 'Asia/Jakarta';
  try {
    userTz = Intl.DateTimeFormat().resolvedOptions().timeZone || 'Asia/Jakarta';
  } catch (e) {}

  return `https://calendar.google.com/calendar/render?action=TEMPLATE&text=${encodeURIComponent(summary)}&dates=${datesParam}&ctz=${encodeURIComponent(userTz)}&details=${encodeURIComponent(desc)}&location=${encodeURIComponent(course)}`;
}

// Sync single event directly to Google Calendar via API or direct web sync
async function syncSingleDeadlineGcal(item, autoOpen = true) {
  const isConnected = !!state.gcalStatus.connected;
  const isOAuthMode = isConnected;

  // If user explicitly configured OAuth mode and is NOT connected, show connect modal
  if (!isOAuthMode && state.gcalStatus.preferredMode === 'oauth' && !isConnected) {
    openGcalConnectModal(true);
    return;
  }

  try {
    let finalItem = item;
    let targetLink = item.gcal_link || '';

    // Direct Web Mode: immediately build template link, save & launch Chrome
    if (!isOAuthMode) {
      targetLink = buildGcalWebUrl(item);
      item.gcal_link = targetLink;
      item.gcal_event_id = item.gcal_event_id || ('direct_' + Date.now());
      item.gcal_synced_at = new Date().toISOString();

      const idx = state.deadlines.findIndex(d => d.id === item.id);
      if (idx !== -1) state.deadlines[idx] = item;
      safeStore.set('notelyDeadlines', state.deadlines);
      if (typeof api === 'function' && engine.ready) {
        api('/deadlines/save', { item }).catch(() => {});
      }
      refreshPage();

      if (autoOpen && targetLink) {
        openExternalUrl(targetLink);
        toast(`Membuka Google Calendar di peramban — Klik Simpan untuk memasukkan tugas.`);
      } else {
        toast(`Jadwal "${item.title}" tersimpan untuk Google Calendar.`);
      }
      return;
    }

    // Official OAuth background API sync
    toast(`Menyinkronkan "${item.title}" ke Google Calendar…`);
    if (typeof api === 'function' && engine.ready) {
      const res = await api('/gcalendar/sync-event', { id: item.id, item });
      if (res.item) {
        finalItem = res.item;
        const idx = state.deadlines.findIndex(d => d.id === item.id);
        if (idx !== -1) state.deadlines[idx] = res.item;
        safeStore.set('notelyDeadlines', state.deadlines);
        refreshPage();
      }
      targetLink = res.gcal?.htmlLink || finalItem.gcal_link || targetLink;
      toast(`✓ "${item.title}" berhasil disinkronkan langsung ke Google Calendar Anda!`);
    } else {
      item.gcal_event_id = 'demo_event_' + Date.now();
      item.gcal_link = 'https://calendar.google.com';
      item.gcal_synced_at = new Date().toISOString();
      const idx = state.deadlines.findIndex(d => d.id === item.id);
      if (idx !== -1) state.deadlines[idx] = item;
      safeStore.set('notelyDeadlines', state.deadlines);
      refreshPage();
      toast(`✓ "${item.title}" berhasil disinkronkan ke Google Calendar (Demo)!`);
    }
  } catch (err) {
    alertError(err);
  }
}


// Sync all active deadlines directly to Google Calendar via API (Two-Way Sync: Pull first, then push)
async function syncAllDeadlinesGcal() {
  if (!state.gcalStatus.connected) {
    openGcalConnectModal(true);
    return;
  }

  try {
    toast('Menyinkronkan dua arah dengan Google Calendar…');
    if (typeof api === 'function' && engine.ready) {
      let pullRes = null;
      try {
        pullRes = await api('/gcalendar/pull', {});
      } catch (pullErr) {
        console.warn('GCal pull error:', pullErr);
      }
      const pushRes = await api('/gcalendar/sync-all', {});
      await loadDeadlinesData();

      const imported = pullRes?.imported || 0;
      const pushed = pushRes?.synced || 0;
      if (imported > 0 && pushed > 0) {
        toast(`✓ Berhasil sinkronisasi 2 arah! (${imported} ditarik dari Google Calendar, ${pushed} terkirim)`);
      } else if (imported > 0) {
        toast(`✓ ${imported} tugas baru dari Google Calendar berhasil ditarik ke Notely!`);
      } else if (pushed > 0) {
        toast(`✓ ${pushed} tugas lokal berhasil dikirim ke Google Calendar!`);
      } else {
        toast('✓ Google Calendar dan Notely sudah sinkron sempurna!');
      }
    } else {
      toast('✓ Semua tugas disinkronkan ke Google Calendar (Mode Demo)!');
    }
    state.gcalMenuOpen = false;
    refreshPage();
  } catch (err) {
    alertError(err);
  }
}

// --------------------------------------------------------------------------
// DEADLINE DATA PERSISTENCE
// --------------------------------------------------------------------------

// Load deadlines from persistent storage
async function loadDeadlinesData() {
  try {
    if (typeof api === 'function' && engine.ready) {
      const data = await api('/deadlines');
      if (Array.isArray(data.deadlines)) {
        state.deadlines = data.deadlines;
        safeStore.set('notelyDeadlines', state.deadlines);
      }

      // If connected to Google Calendar and has mock direct_ items, auto-sync them in background with gentle pacing
      if (state.gcalStatus.connected && state.gcalStatus.autoSync !== false) {
        const hasUnsynced = state.deadlines.some(d => d.status !== 'completed' && (!d.gcal_event_id || String(d.gcal_event_id).startsWith('direct_')));
        if (hasUnsynced && !window.__autoSyncedDeadlinesOnce) {
          window.__autoSyncedDeadlinesOnce = true;
          setTimeout(() => {
            api('/gcalendar/sync-all', {}).then(res => {
              if (res && res.synced) {
                api('/deadlines').then(fresh => {
                  if (Array.isArray(fresh.deadlines)) {
                    state.deadlines = fresh.deadlines;
                    safeStore.set('notelyDeadlines', state.deadlines);
                    refreshPage();
                  }
                }).catch(() => {});
              }
            }).catch(() => {});
          }, 1500);
        }
      }
    } else {
      const local = safeStore.get('notelyDeadlines');
      if (Array.isArray(local)) state.deadlines = local;
    }
  } catch (e) {
    const local = safeStore.get('notelyDeadlines');
    if (Array.isArray(local)) state.deadlines = local;
  }
}

// Save a single deadline
async function persistDeadline(item) {
  try {
    if (typeof api === 'function' && engine.ready) {
      const res = await api('/deadlines/save', item);
      item = res.deadline || item;
    }
    const idx = state.deadlines.findIndex(d => d.id === item.id);
    if (idx !== -1) state.deadlines[idx] = item;
    else state.deadlines.unshift(item);
    safeStore.set('notelyDeadlines', state.deadlines);

    // If backend didn't already sync it and Google Calendar is connected, sync event
    if (state.gcalStatus.connected && state.gcalStatus.autoSync !== false && !item.gcal_event_id) {
      syncSingleDeadlineGcal(item, false);
    }
    return item;
  } catch (e) {
    const idx = state.deadlines.findIndex(d => d.id === item.id);
    if (idx !== -1) state.deadlines[idx] = item;
    else state.deadlines.unshift(item);
    safeStore.set('notelyDeadlines', state.deadlines);
    return item;
  }
}

// Batch save deadlines (e.g. from AI extraction)
async function persistDeadlinesBatch(items) {
  try {
    if (typeof api === 'function' && engine.ready) {
      const res = await api('/deadlines/batch', { items });
      if (Array.isArray(res.deadlines)) items = res.deadlines;
    }
    for (const it of items) {
      const idx = state.deadlines.findIndex(d => d.id === it.id);
      if (idx !== -1) state.deadlines[idx] = it;
      else state.deadlines.unshift(it);
    }
    safeStore.set('notelyDeadlines', state.deadlines);
  } catch (e) {
    for (const it of items) {
      const idx = state.deadlines.findIndex(d => d.id === it.id);
      if (idx !== -1) state.deadlines[idx] = it;
      else state.deadlines.unshift(it);
    }
    safeStore.set('notelyDeadlines', state.deadlines);
  }
}

// Delete deadline
async function removeDeadline(id) {
  state.deadlines = state.deadlines.filter(d => d.id !== id);
  safeStore.set('notelyDeadlines', state.deadlines);
  try {
    if (typeof api === 'function' && engine.ready) {
      await api('/deadlines/delete', { id });
    }
  } catch (e) {}
}

// Confirmation dialog for deleting a task (instant single-click + modal safety)
function promptDeleteDeadline(id) {
  const item = state.deadlines.find(d => d.id === id);
  const title = item ? item.title : 'tugas ini';

  const content = `
    <div style="font-size:13.5px;color:#435c4e;line-height:1.6;padding:4px 0;">
      <p style="margin:0 0 12px;">Apakah Anda yakin ingin menghapus tugas <strong>"${esc(title)}"</strong>?</p>
      <div style="background:#fdf2f2;border:1px solid #f9d5d8;border-radius:10px;padding:10px 14px;color:#ba3939;font-size:12px;display:flex;align-items:center;gap:8px;">
        <span style="display:inline-flex;color:#ba3939">${icon('trash')}</span>
        <span>Tugas yang telah dihapus tidak dapat dipulihkan kembali.</span>
      </div>
    </div>
  `;

  window.__deadlineIdToDelete = id;

  modal('Hapus Tenggat Tugas', content, [
    ['Batal', 'close-modal', 'ghost'],
    ['Hapus Tugas', 'confirm-delete-deadline', 'danger']
  ]);
}

// Confirmation dialog for completing & removing a task with delightful celebration animation
function promptCompleteAndRemoveDeadline(id) {
  const d = state.deadlines.find(x => x.id === id);
  if (!d) return;

  const content = `
    <div style="display:flex;flex-direction:column;gap:14px;padding:4px 0">
      <div style="display:flex;align-items:center;gap:12px;background:#f0fdf4;border:1px solid #bbf7d0;padding:12px 14px;border-radius:12px">
        <div style="width:36px;height:36px;border-radius:50%;background:#22c55e;color:#fff;display:grid;place-items:center;font-size:18px;font-weight:700">✓</div>
        <div>
          <h4 style="margin:0 0 2px;font-size:14px;color:#15803d;font-weight:700">Tandai Tugas Selesai?</h4>
          <p style="margin:0;font-size:12px;color:#166534">Tugas yang diselesaikan akan otomatis dibersihkan dari jadwal kalender.</p>
        </div>
      </div>
      <p style="font-size:13.5px;color:#374151;margin:0;line-height:1.6">
        Apakah Anda menyatakan tugas <strong>"${esc(d.title)}"</strong> (${esc(d.course || 'Umum')}) telah selesai?
      </p>
      <div style="font-size:12px;color:#6b7280;background:#f9fafb;padding:10px 12px;border-radius:8px;border:1px solid #e5e7eb">
        ✓ Tugas akan dituntaskan dan otomatis dihapus dari daftar tenggat agar kalender Anda tetap bersih dan terorganisir.
      </div>
    </div>
  `;

  window.__deadlineIdToComplete = id;

  modal('Konfirmasi Tugas Selesai', content, [
    ['Batal', 'cancel-complete-deadline', 'ghost'],
    ['✓ Ya, Selesaikan & Hapus', 'confirm-complete-deadline', 'accent']
  ]);
}

async function completeAndRemoveDeadlineWithAnimation(id) {
  const d = state.deadlines.find(x => x.id === id);
  closeModal();
  if (!d) return;

  // Add smooth completion animation on card
  const card = document.querySelector(`.deadline-card input[data-id="${id}"]`)?.closest('.deadline-card') ||
               document.querySelector(`.cal-modal-task-item input[data-id="${id}"]`)?.closest('.cal-modal-task-item');
  if (card) {
    card.classList.add('task-completing-anim');
  }

  setTimeout(async () => {
    // Delete from Notely & Google Calendar
    await removeDeadline(id);
    if (state.selectedDeadlineDate) {
      const remaining = state.deadlines.filter(x => x.deadline === state.selectedDeadlineDate);
      if (!remaining.length) state.selectedDeadlineDate = null;
    }
    toast(`🎉 Hebat! Tugas "${d.title}" selesai & dibersihkan.`);
    refreshPage();
  }, 360);
}

// Toggle completion status
async function toggleDeadline(id) {
  const d = state.deadlines.find(x => x.id === id);
  if (!d) return;
  d.status = d.status === 'completed' ? 'pending' : 'completed';
  try {
    if (typeof api === 'function' && engine.ready) {
      await api('/deadlines/toggle', { id });
    }
  } catch (e) {}
  safeStore.set('notelyDeadlines', state.deadlines);
  refreshPage();
}

// Helper: Calculate countdown / relative status
function getDeadlineStatus(deadlineStr, timeStr, status) {
  if (status === 'completed') return { label: 'Selesai', tagClass: 'tag-done', urgent: false };
  if (!deadlineStr) return { label: 'Tanpa tanggal', tagClass: 'tag-neutral', urgent: false };

  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const parts = deadlineStr.split('-').map(Number);
  const target = new Date(parts[0], parts[1] - 1, parts[2]);
  target.setHours(0, 0, 0, 0);

  const diffMs = target.getTime() - today.getTime();
  const diffDays = Math.round(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays < 0) return { label: `Terlewat ${Math.abs(diffDays)} hari`, tagClass: 'tag-overdue', urgent: true, isOverdue: true };
  if (diffDays === 0) return { label: `Hari ini (${timeStr || '23:59'})`, tagClass: 'tag-today', urgent: true, isToday: true };
  if (diffDays === 1) return { label: `Besok (${timeStr || '23:59'})`, tagClass: 'tag-soon', urgent: true, isSoon: true };
  if (diffDays <= 3) return { label: `${diffDays} hari lagi`, tagClass: 'tag-soon', urgent: true, isSoon: true };
  return { label: `${diffDays} hari lagi`, tagClass: 'tag-normal', urgent: false };
}

// Helper: Format Indonesian Date
function formatIndoDate(dateStr) {
  if (!dateStr) return '-';
  const months = ['Januari','Februari','Maret','April','Mei','Juni','Juli','Agustus','September','Oktober','November','Desember'];
  const parts = dateStr.split('-').map(Number);
  if (parts.length !== 3) return dateStr;
  return `${parts[2]} ${months[parts[1] - 1]} ${parts[0]}`;
}

// Helper: Export all active deadlines to iCalendar (.ics)
function exportDeadlinesICS() {
  const activeTasks = state.deadlines.filter(d => d.status !== 'completed');
  if (!activeTasks.length) {
    toast('Tidak ada tugas aktif untuk diekspor ke kalender.');
    return;
  }

  let ics = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Notely Desktop//Deadline Notes//ID\r\nCALSCALE:GREGORIAN\r\nMETHOD:PUBLISH\r\nX-WR-CALNAME:Notely Tenggat Tugas\r\n";
  for (const d of activeTasks) {
    const dateClean = (d.deadline || '').replace(/-/g, '');
    const timeClean = (d.time || '23:59').replace(/:/g, '') + '00';
    const uid = (d.id || Math.random().toString(36).substring(2)) + '@notely.desktop';
    const cleanDesc = (d.description || '').replace(/\r?\n/g, '\\n');
    ics += `BEGIN:VEVENT\r\nUID:${uid}\r\nDTSTAMP:${dateClean}T${timeClean}Z\r\nDTSTART:${dateClean}T${timeClean}\r\nDTEND:${dateClean}T${timeClean}\r\nSUMMARY:${d.title} [${d.course || 'Tugas'}]\r\nDESCRIPTION:${cleanDesc}\r\nSTATUS:CONFIRMED\r\nPRIORITY:${d.priority === 'tinggi' ? '1' : d.priority === 'sedang' ? '5' : '9'}\r\nBEGIN:VALARM\r\nTRIGGER:-PT1D\r\nACTION:DISPLAY\r\nDESCRIPTION:Pengingat Tenggat Tugas: ${d.title}\r\nEND:VALARM\r\nEND:VEVENT\r\n`;
  }
  ics += "END:VCALENDAR\r\n";

  const blob = new Blob([ics], { type: 'text/calendar;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `notely_deadlines_${new Date().toISOString().slice(0, 10)}.ics`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  setTimeout(() => URL.revokeObjectURL(url), 1000);
  toast('File kalender (.ics) berhasil diunduh.');
}

// --------------------------------------------------------------------------
// MAIN PAGE RENDERING: deadlinesPage()
// --------------------------------------------------------------------------
function deadlinesPage() {
  checkGcalPromptOnEnter();

  // Automatic Two-Way Sync from Google Calendar (events created/updated from smartphone)
  const nowPull = Date.now();
  if (state.gcalStatus.connected && (!window.__lastGcalPullTime || nowPull - window.__lastGcalPullTime > 15000)) {
    window.__lastGcalPullTime = nowPull;
    if (typeof api === 'function' && engine.ready) {
      api('/gcalendar/pull', {}).then(res => {
        if (res && (res.imported > 0 || res.updated > 0)) {
          loadDeadlinesData().then(() => {
            if (state.page === 'deadlines') refreshPage();
          });
        }
      }).catch(() => {});
    }
  }

  // Hide floating recording dock on deadlines page so it doesn't obstruct content
  const dock = document.getElementById('recording-dock');
  if (dock) {
    dock.classList.add('dock-hidden-deadlines');
    dock.hidden = true;
  }
  if (typeof applyDockScale === 'function') {
    applyDockScale();
  }

  // Ensure calendar view fills full height without empty dock padding
  const mainPage = document.getElementById('main-page');
  if (mainPage) {
    if (state.deadlineView === 'calendar') {
      mainPage.style.paddingBottom = '0px';
      mainPage.style.overflow = 'hidden';
    } else {
      mainPage.style.paddingBottom = '';
      mainPage.style.overflow = '';
    }
  }

  const all = state.deadlines || [];
  const q = (state.deadlineSearch || '').toLowerCase().trim();
  const filtered = all.filter(d => {
    const matchQuery = !q || (d.title + ' ' + (d.course || '') + ' ' + (d.description || '')).toLowerCase().includes(q);
    if (!matchQuery) return false;
    const st = getDeadlineStatus(d.deadline, d.time, d.status);
    if (state.deadlineFilter === 'upcoming') return d.status !== 'completed' && !st.isOverdue;
    if (state.deadlineFilter === 'urgent') return d.status !== 'completed' && st.urgent;
    if (state.deadlineFilter === 'completed') return d.status === 'completed';
    if (state.deadlineFilter === 'overdue') return d.status !== 'completed' && st.isOverdue;
    return true;
  });

  const countActive = all.filter(d => d.status !== 'completed').length;
  const countUrgent = all.filter(d => d.status !== 'completed' && getDeadlineStatus(d.deadline, d.time, d.status).urgent).length;
  const isGcalConnected = !!state.gcalStatus.connected;
  const isWebMode = state.gcalStatus.preferredMode === 'web';
  // Privacy setting: User can hide/show the status indicator in settings
  const showIndicator = state.settings.showGcalIndicator !== false;

  const isCalView = state.deadlineView === 'calendar';
  return `<div class="deadlines-page ${isCalView ? 'calendar-view-fixed' : ''}">
    <div class="deadlines-top-bar">
      <div class="deadlines-title-area">
        <span class="eyebrow">AGENDA & MANAJEMEN TUGAS</span>
        <h1>Tenggat Tugas</h1>
        <p>${countActive} tugas aktif · ${countUrgent ? `<strong class="urgent-text">${countUrgent} mendesak</strong>` : 'Semua terkendali'}</p>
      </div>

      <div class="deadlines-actions">
        <!-- Google Calendar Indicator (Configurable in Settings: hidden by default or toggleable) -->
        ${showIndicator ? (isGcalConnected ? `
          <div style="display:flex;align-items:center;gap:8px">
            <button class="button small ghost gcal-quick-sync-btn" data-action="gcal-sync-all" title="Sinkronkan dua arah sekarang dengan Google Calendar">
              ${icon('bolt')} Sinkronkan
            </button>
            <div class="gcal-popover-wrap">
              <button class="gcal-status-pill connected" data-action="toggle-gcal-menu" title="Google Calendar: Tersambung (Klik untuk opsi)">
                <span class="gcal-dot green"></span>
                <span>Google Calendar</span>
                ${icon('chevron')}
              </button>
              ${state.gcalMenuOpen ? renderGcalDropdownMenu() : ''}
            </div>
          </div>
        ` : isWebMode ? `
          <div class="gcal-popover-wrap">
            <button class="gcal-status-pill web-mode" data-action="toggle-gcal-menu" title="Google Calendar: Mode Cepat Web (Klik untuk opsi)">
              <span class="gcal-dot blue"></span>
              <span>Google Calendar (Web)</span>
              ${icon('chevron')}
            </button>
            ${state.gcalMenuOpen ? renderGcalDropdownMenu() : ''}
          </div>
        ` : `
          <button class="gcal-status-pill disconnected" data-action="open-gcal-connect-modal" title="Google Calendar: Belum tersambung (Klik untuk opsi)">
            <span class="gcal-dot red"></span>
            <span>Google Calendar</span>
          </button>
        `) : ''}
      </div>
    </div>

    <div class="deadlines-toolbar">
      <div class="deadlines-toolbar-row">
        <div class="segmented" role="tablist" aria-label="Mode Tampilan">
          <button class="${state.deadlineView === 'calendar' ? 'active' : ''}" data-action="switch-deadline-view" data-id="calendar">${icon('calendar')} Kalender</button>
          <button class="${state.deadlineView === 'list' ? 'active' : ''}" data-action="switch-deadline-view" data-id="list">${icon('notes')} Daftar Tugas</button>
        </div>

        ${isCalView ? `
          <div class="deadlines-toolbar-actions">
            <button class="button accent" data-action="open-deadline-ai">${icon('spark')} Buat dengan AI</button>
          </div>
        ` : `
          <div class="deadlines-search-wrap">
            ${icon('search')}
            <input type="text" id="deadline-search-input" placeholder="Cari nama tugas atau mata kuliah…" value="${esc(state.deadlineSearch)}" autocomplete="off">
            ${state.deadlineSearch ? `<button class="search-clear" data-action="clear-deadline-search">×</button>` : ''}
          </div>
        `}
      </div>

      ${state.deadlineView === 'list' ? `
      <div class="deadline-filter-pills">
        <button class="filter-pill ${state.deadlineFilter === 'all' ? 'active' : ''}" data-action="filter-deadline" data-id="all">Semua (${all.length})</button>
        <button class="filter-pill ${state.deadlineFilter === 'upcoming' ? 'active' : ''}" data-action="filter-deadline" data-id="upcoming">Mendatang</button>
        <button class="filter-pill ${state.deadlineFilter === 'urgent' ? 'active' : ''}" data-action="filter-deadline" data-id="urgent">Mendesak (${countUrgent})</button>
        <button class="filter-pill ${state.deadlineFilter === 'overdue' ? 'active' : ''}" data-action="filter-deadline" data-id="overdue">Terlewat</button>
        <button class="filter-pill ${state.deadlineFilter === 'completed' ? 'active' : ''}" data-action="filter-deadline" data-id="completed">Selesai</button>
      </div>` : ''}
    </div>

    <div class="deadlines-body">
      ${state.deadlineView === 'calendar' ? renderDeadlineCalendar() : renderDeadlineList(filtered)}
    </div>

    ${state.selectedDeadlineDate ? renderDateDetailsModal(state.selectedDeadlineDate) : ''}
  </div>`;
}

// --------------------------------------------------------------------------
// GOOGLE CALENDAR DROPDOWN MENU (Clean & Private)
// --------------------------------------------------------------------------
function renderGcalDropdownMenu() {
  const isConnected = !!state.gcalStatus.connected;
  const isWebMode = state.gcalStatus.preferredMode === 'web';
  const isAutoSync = state.gcalStatus.autoSync !== false;
  const rawEmail = state.gcalStatus.email || '';
  const displayEmail = state.showGcalEmail ? rawEmail : maskEmail(rawEmail);

  if (isWebMode && !isConnected) {
    return `<div class="gcal-dropdown-menu" onclick="event.stopPropagation()">
      <div class="gcal-menu-header">
        <div class="space-between" style="margin-bottom:4px">
          <span class="gcal-status-badge-inline" style="background:#e8f4fd;color:#185a9d;border:1px solid #c7e1f4">
            <span class="gcal-dot blue"></span> Mode Cepat Web
          </span>
        </div>
        <small>METODE SINKRONISASI</small>
        <strong>Google Calendar (Tanpa Setup)</strong>
      </div>

      <div class="gcal-menu-actions">
        <button class="button small" data-action="open-gcal-web-test">
          ${icon('google')} Buka Google Calendar di Browser
        </button>
        <button class="button small ghost" data-action="open-gcal-creds-modal">
          ${icon('sliders')} Kredensial Khusus Developer
        </button>
        <button class="button small ghost" data-action="go-gcal-settings">
          ${icon('settings')} Pengaturan Integrasi
        </button>
      </div>
    </div>`;
  }

  return `<div class="gcal-dropdown-menu" onclick="event.stopPropagation()">
    <div class="gcal-menu-header">
      <div class="space-between" style="margin-bottom:4px">
        <span class="gcal-status-badge-inline connected">
          <span class="gcal-dot green"></span> Tersambung
        </span>
        ${rawEmail ? `
          <button class="button small ghost icon-only" data-action="toggle-mask-email" title="${state.showGcalEmail ? 'Sembunyikan email' : 'Tampilkan email'}" style="width:26px;height:26px">
            ${icon(state.showGcalEmail ? 'eyeoff' : 'eye')}
          </button>
        ` : ''}
      </div>
      <small>STATUS INTEGRASI</small>
      <strong>${esc(displayEmail || 'Google Calendar Aktif')}</strong>
    </div>

    <div class="gcal-menu-row">
      <span>Sinkronkan tugas otomatis</span>
      <input type="checkbox" class="switch" ${isAutoSync ? 'checked' : ''} data-action="toggle-gcal-autosync" aria-label="Auto sync">
    </div>

    <div class="gcal-menu-actions">
      <button class="button small" data-action="gcal-sync-all">
        ${icon('bolt')} Sinkronkan Semua Tugas Sekarang
      </button>
      <button class="button small ghost" data-action="choose-gcal-web-mode">
        ${icon('globe')} Beralih ke Mode Cepat Web
      </button>
      <button class="button small ghost" data-action="go-gcal-settings">
        ${icon('settings')} Pengaturan di Settings
      </button>
      <button class="button small ghost danger-hover" style="color:var(--danger)" data-action="gcal-disconnect">
        ${icon('trash')} Putuskan Sambungan
      </button>
    </div>
  </div>`;
}

// --------------------------------------------------------------------------
// SETTINGS PAGE CARD: renderGcalSettingsCard() (For Settings -> General)
// --------------------------------------------------------------------------
function renderGcalSettingsCard() {
  const isConnected = !!state.gcalStatus.connected;
  const rawEmail = state.gcalStatus.email || '';
  const isAutoSync = state.gcalStatus.autoSync !== false;
  const showIndicator = state.settings.showGcalIndicator !== false;
  const displayEmail = state.showGcalEmail ? rawEmail : maskEmail(rawEmail);

  return `<div class="setting-card gcal-settings-card">
    <!-- Card Header -->
    <div class="gcal-card-header">
      <div class="gcal-card-title-group">
        <div class="gcal-icon-badge">${icon('google')}</div>
        <div>
          <h3 class="section-title">Integrasi Google Calendar</h3>
          <p class="section-description">Jadwalkan tenggat tugas kuliah dan pengingat ke Google Calendar.</p>
        </div>
      </div>
      <div class="gcal-status-pill-badge ${isConnected ? 'connected' : 'web-active'}">
        <span class="gcal-dot ${isConnected ? 'green' : 'blue'}"></span>
        <span>${isConnected ? 'Kredensial Khusus Terhubung' : 'Mode Cepat Web (Aktif)'}</span>
      </div>
    </div>

    ${isConnected ? `
      <!-- State: Akun Terhubung via Kredensial Khusus -->
      <div class="gcal-connected-box">
        <div class="gcal-profile-row">
          <div class="gcal-profile-info">
            <div class="gcal-avatar-circle">${icon('google')}</div>
            <div class="gcal-profile-text">
              <span class="gcal-profile-label">Kredensial Developer Google Cloud</span>
              <div class="gcal-email-line">
                <strong>${esc(displayEmail || 'Google Calendar Terhubung')}</strong>
                ${rawEmail ? `
                  <button class="button small ghost icon-only" data-action="toggle-mask-email" title="${state.showGcalEmail ? 'Sembunyikan email' : 'Tampilkan email'}" style="width:28px;height:28px">
                    ${icon(state.showGcalEmail ? 'eyeoff' : 'eye')}
                  </button>
                ` : ''}
              </div>
              <span class="gcal-active-tag">✓ Sinkronisasi latar belakang aktif via API Google</span>
            </div>
          </div>
          <button class="button small ghost danger-hover" data-action="gcal-disconnect" style="color:var(--danger)">
            ${icon('trash')} Putuskan Akun
          </button>
        </div>

        <div class="gcal-setting-toggle-card">
          <div class="gcal-toggle-text">
            <h4>Sinkronisasi Otomatis Tugas Baru</h4>
            <p>Setiap tugas baru dari prompt AI, tangkapan layar, atau tambah manual langsung dikirim ke kalender secara otomatis.</p>
          </div>
          <input type="checkbox" class="switch" ${isAutoSync ? 'checked' : ''} data-action="toggle-gcal-autosync" aria-label="Auto sync">
        </div>

        <div class="gcal-actions-bar">
          <button class="button small" data-action="gcal-sync-all">${icon('bolt')} Sinkronkan Semua Tugas Sekarang</button>
          <button class="button small ghost" data-action="open-gcal-creds-modal">${icon('sliders')} Atur Ulang Kredensial</button>
          <button class="button small ghost" data-action="choose-gcal-web-mode">${icon('globe')} Gunakan Mode Cepat Web</button>
        </div>
      </div>
    ` : `
      <!-- State Default: Mode Cepat Web (Bersih, Rapi & Ringkas) -->
      <div class="gcal-clean-panel">
        <div class="gcal-clean-row">
          <div class="gcal-avatar-circle" style="background:#edf7f2;color:#18786c">${icon('globe')}</div>
          <div style="flex:1">
            <h4 style="font-size:14.5px;font-weight:700;color:#183226;margin:0 0 4px">Mode Cepat Web (1-Klik Tanpa Setup)</h4>
            <p style="font-size:12.5px;color:#5a7465;line-height:1.6;margin:0">
              Setiap tugas yang Anda simpan atau sinkronkan akan langsung membuka tab peramban ke halaman resmi Google Calendar dengan judul, tanggal, dan jam yang sudah terisi otomatis. Anda tinggal menekan tombol <strong>Simpan</strong> satu kali di peramban.
            </p>
          </div>
        </div>

        <div class="gcal-clean-actions">
          <button class="button small accent" data-action="open-gcal-web-test">
            ${icon('google')} Buka Google Calendar di Browser
          </button>
          <button class="button small ghost" data-action="open-gcal-creds-modal" style="color:#4a6454">
            ${icon('sliders')} Gunakan Kredensial Khusus Developer
          </button>
        </div>
      </div>
    `}

    <!-- Display Settings Sub-Row -->
    <div class="setting-row" style="margin-top:20px;padding-top:16px;border-top:1px solid #e7ede9">
      <div>
        <h4 style="font-size:13.5px;font-weight:600;color:#22362b;margin:0 0 4px">Tampilkan Indikator Status di Header</h4>
        <p style="font-size:12px;color:#718579;margin:0">Tampilkan pill status Google Calendar di bagian atas halaman Tenggat Tugas.</p>
      </div>
      <input type="checkbox" class="switch" ${showIndicator ? 'checked' : ''} data-action="toggle-header-indicator" aria-label="Toggle indicator">
    </div>
  </div>`;
}

// --------------------------------------------------------------------------
// CALENDAR VIEW: renderDeadlineCalendar()
// --------------------------------------------------------------------------
function renderDeadlineCalendar() {
  const months = ['Januari','Februari','Maret','April','Mei','Juni','Juli','Agustus','September','Oktober','November','Desember'];
  const curYear = state.calendarYear;
  const curMonth = state.calendarMonth;

  // First day of current month
  const firstDay = new Date(curYear, curMonth, 1);
  let startDay = firstDay.getDay() - 1;
  if (startDay === -1) startDay = 6;

  const daysInMonth = new Date(curYear, curMonth + 1, 0).getDate();
  const daysInPrevMonth = new Date(curYear, curMonth, 0).getDate();

  const today = new Date();
  const todayStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`;

  const cells = [];
  // Previous month trailing days
  for (let i = startDay - 1; i >= 0; i--) {
    const dayNum = daysInPrevMonth - i;
    const prevMonthIdx = curMonth === 0 ? 11 : curMonth - 1;
    const prevYear = curMonth === 0 ? curYear - 1 : curYear;
    const dateStr = `${prevYear}-${String(prevMonthIdx + 1).padStart(2, '0')}-${String(dayNum).padStart(2, '0')}`;
    cells.push({ dayNum, dateStr, isCurrentMonth: false });
  }

  // Current month days
  for (let day = 1; day <= daysInMonth; day++) {
    const dateStr = `${curYear}-${String(curMonth + 1).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
    cells.push({ dayNum: day, dateStr, isCurrentMonth: true, isToday: dateStr === todayStr });
  }

  // Next month leading days to complete full grid
  const remaining = (7 - (cells.length % 7)) % 7;
  for (let day = 1; day <= remaining; day++) {
    const nextMonthIdx = curMonth === 11 ? 0 : curMonth + 1;
    const nextYear = curMonth === 11 ? curYear + 1 : curYear;
    const dateStr = `${nextYear}-${String(nextMonthIdx + 1).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
    cells.push({ dayNum: day, dateStr, isCurrentMonth: false });
  }

  // Map deadlines by date
  const deadlineMap = new Map();
  for (const d of state.deadlines) {
    if (!d.deadline) continue;
    if (!deadlineMap.has(d.deadline)) deadlineMap.set(d.deadline, []);
    deadlineMap.get(d.deadline).push(d);
  }

  const numWeeks = Math.ceil(cells.length / 7) || 5;

  return `<div class="calendar-wrapper" style="--cal-rows: ${numWeeks};">
    <div class="calendar-header-bar">
      <div class="calendar-month-title">
        <h2>${months[curMonth]} ${curYear}</h2>
        <button class="button small ghost" data-action="cal-today">Hari ini</button>
      </div>
      <div class="calendar-nav-buttons">
        <button class="cal-nav-btn" data-action="cal-prev" title="Bulan Sebelumnya">${icon('chevron')}</button>
        <button class="cal-nav-btn next" data-action="cal-next" title="Bulan Selanjutnya">${icon('chevron')}</button>
      </div>
    </div>

    <div class="calendar-days-bar">
      <div class="calendar-day-header">Sen</div>
      <div class="calendar-day-header">Sel</div>
      <div class="calendar-day-header">Rab</div>
      <div class="calendar-day-header">Kam</div>
      <div class="calendar-day-header">Jum</div>
      <div class="calendar-day-header weekend">Sab</div>
      <div class="calendar-day-header weekend">Min</div>
    </div>

    <div class="calendar-cells-grid" style="grid-template-rows: repeat(${numWeeks}, minmax(0, 1fr));">
      ${cells.map(c => {
        const tasks = deadlineMap.get(c.dateStr) || [];
        const hasTasks = tasks.length > 0;
        const activeTasks = tasks.filter(t => t.status !== 'completed');
        const hasUrgent = activeTasks.some(t => getDeadlineStatus(t.deadline, t.time, t.status).urgent);

        return `<div class="calendar-cell ${c.isCurrentMonth ? '' : 'outside-month'} ${c.isToday ? 'is-today' : ''} ${hasTasks ? 'has-tasks' : ''}" data-action="select-cal-date" data-id="${c.dateStr}">
          <div class="cell-top">
            <span class="day-number ${c.isToday ? 'today-badge' : ''}">${c.dayNum}</span>
            ${hasUrgent ? `<span class="urgent-dot" title="Ada tugas mendesak!"></span>` : ''}
          </div>
          <div class="cell-task-pills">
            ${tasks.slice(0, 2).map(t => {
              const st = getDeadlineStatus(t.deadline, t.time, t.status);
              return `<div class="task-pill ${t.status === 'completed' ? 'pill-done' : st.isOverdue ? 'pill-overdue' : st.urgent ? 'pill-urgent' : 'pill-normal'}" title="${esc(t.title)} (${esc(t.course || 'Tugas')})">
                ${t.status === 'completed' ? '✓ ' : ''}${esc(t.title)}
              </div>`;
            }).join('')}
            ${tasks.length > 2 ? `<div class="task-pill-more">+${tasks.length - 2} tugas lagi</div>` : ''}
          </div>
        </div>`;
      }).join('')}
    </div>
  </div>`;
}

// --------------------------------------------------------------------------
// LIST VIEW: renderDeadlineList(tasks) (Clean, Elegant & Balanced)
// --------------------------------------------------------------------------
function renderDeadlineList(tasks) {
  if (!tasks.length) {
    return `<div class="deadline-empty-box">
      <div class="deadline-empty-icon">${icon('calendar')}</div>
      <h3>Belum ada tenggat tugas</h3>
      <p>Mulai catat jadwal tugas Anda. Gunakan AI untuk mengekstrak tugas otomatis dari tangkapan layar/prompt pengumuman, atau tambahkan secara manual.</p>
      <div class="deadline-empty-actions">
        <button class="button accent small" data-action="open-deadline-ai">${icon('spark')} Buat dengan AI</button>
        <button class="button small" data-action="open-manual-deadline">${icon('plus')} Tambah manual</button>
      </div>
    </div>`;
  }

  // Sort tasks: pending first, then by deadline date ascending
  const sorted = [...tasks].sort((a, b) => {
    if (a.status === 'completed' && b.status !== 'completed') return 1;
    if (a.status !== 'completed' && b.status === 'completed') return -1;
    return (a.deadline || '').localeCompare(b.deadline || '');
  });

  const isGcalConnected = !!state.gcalStatus.connected;

  return `<div class="deadlines-list-container">
    ${sorted.map(d => {
      const st = getDeadlineStatus(d.deadline, d.time, d.status);
      const isDone = d.status === 'completed';
      const prioClass = d.priority === 'tinggi' ? 'prio-high' : d.priority === 'rendah' ? 'prio-low' : 'prio-med';
      const hasGcal = !!d.gcal_event_id;

      return `<div class="deadline-card ${isDone ? 'is-done' : ''} ${st.isOverdue ? 'is-overdue' : ''}">
        <div class="card-checkbox-area">
          <input type="checkbox" class="task-check" ${isDone ? 'checked' : ''} data-action="toggle-deadline" data-id="${esc(d.id)}" aria-label="Tandai selesai">
        </div>
        <div class="card-content-area">
          <div class="card-header-row">
            <h3 class="task-title">${esc(d.title)}</h3>
            <div class="card-tags">
              ${d.course ? `<span class="course-badge">${esc(d.course)}</span>` : ''}
              <span class="prio-badge ${prioClass}">${esc(d.priority || 'sedang')}</span>
              <span class="status-badge ${st.tagClass}">${esc(st.label)}</span>
              ${hasGcal ? `<span class="gcal-badge-synced" title="Tersinkron otomatis di Google Calendar">${icon('check')} Google Calendar</span>` : ''}
            </div>
          </div>
          ${d.description ? `<p class="task-desc">${esc(d.description)}</p>` : ''}
          <div class="card-meta-row">
            <span class="meta-item">${icon('calendar')} <strong>${formatIndoDate(d.deadline)}</strong> · ${esc(d.time || '23:59')} WIB</span>
          </div>
        </div>
        <div class="card-actions-area">
          ${hasGcal ? `
            <button class="button small ghost" data-action="open-gcal-link" data-id="${esc(d.gcal_link || 'https://calendar.google.com')}" title="Buka di Google Calendar">${icon('google')} Buka di Kalender</button>
          ` : isGcalConnected ? `
            <button class="button small accent-soft" data-action="sync-single-gcal" data-id="${esc(d.id)}" title="Kirim langsung ke Google Calendar di latar belakang">
              ${icon('google')} Sync Kalender
            </button>
          ` : (state.gcalStatus.preferredMode === 'web' || !state.gcalStatus.hasCredentials) ? `
            <button class="button small accent-soft" data-action="sync-single-gcal" data-id="${esc(d.id)}" title="Buka dan simpan langsung di Google Calendar">
              ${icon('google')} Buka di Kalender
            </button>
          ` : `
            <button class="button small google-btn" data-action="open-gcal-connect-modal" title="Sambungkan Google Calendar untuk sinkronisasi otomatis">
              ${icon('google')} Sambungkan Kalender
            </button>
          `}
          <button class="button small ghost" data-action="edit-deadline" data-id="${esc(d.id)}" title="Edit tugas">${icon('edit')}</button>
          <button class="button small ghost danger-hover" data-action="delete-deadline" data-id="${esc(d.id)}" title="Hapus tugas">${icon('trash')}</button>
        </div>
      </div>`;
    }).join('')}
  </div>`;
}

// --------------------------------------------------------------------------
// DATE DETAILS MODAL (when clicking a calendar cell)
// --------------------------------------------------------------------------
function renderDateDetailsModal(dateStr) {
  const tasks = state.deadlines.filter(d => d.deadline === dateStr);
  const formattedDate = formatIndoDate(dateStr);

  return `<div class="overlay" data-action="close-cal-details">
    <div class="modal cal-details-modal" onclick="event.stopPropagation()">
      <div class="modal-head-row">
        <div>
          <span class="eyebrow">TENGGAT TANGGAL</span>
          <h2>${formattedDate}</h2>
        </div>
        <button class="dismiss" data-action="close-cal-details">×</button>
      </div>

      <div class="cal-modal-tasks">
        ${tasks.length ? tasks.map(t => {
          const st = getDeadlineStatus(t.deadline, t.time, t.status);
          const isDone = t.status === 'completed';
          const hasGcal = !!t.gcal_event_id;

          return `<div class="cal-modal-task-item ${isDone ? 'done' : ''}">
            <input type="checkbox" class="task-check" ${isDone ? 'checked' : ''} data-action="toggle-deadline" data-id="${esc(t.id)}">
            <div class="cal-modal-task-info">
              <strong class="task-title">${esc(t.title)}</strong>
              <div class="task-sub">
                ${t.course ? `<span>${esc(t.course)}</span> · ` : ''}
                <span>Jam ${esc(t.time || '23:59')} WIB</span>
                ${hasGcal ? ` · <span style="color:#1b7440;font-weight:600">✓ Google Calendar</span>` : ''}
              </div>
              ${t.description ? `<p class="task-desc-compact">${esc(t.description)}</p>` : ''}
            </div>
            <div class="cal-modal-task-actions">
              ${hasGcal ? `
                <button class="button small ghost icon-only" data-action="open-gcal-link" data-id="${esc(t.gcal_link || 'https://calendar.google.com')}" title="Buka di Google Calendar">${icon('google')}</button>
              ` : `
                <button class="button small accent-soft icon-only" data-action="sync-single-gcal" data-id="${esc(t.id)}" title="Simpan ke Google Calendar">${icon('google')}</button>
              `}
              <button class="button small ghost icon-only danger-hover" data-action="delete-deadline" data-id="${esc(t.id)}" title="Hapus">${icon('trash')}</button>
            </div>
          </div>`;
        }).join('') : `<p class="no-tasks-hint">Belum ada tugas pada tanggal ini.</p>`}
      </div>

      <div class="modal-footer-row">
        <button class="button small accent" data-action="open-manual-deadline-date" data-id="${esc(dateStr)}">${icon('plus')} Tambah tugas pada tanggal ini</button>
        <button class="button small" data-action="close-cal-details">Tutup</button>
      </div>
    </div>
  </div>`;
}

// --------------------------------------------------------------------------
// MODAL: AI EXTRACTOR (Prompt AI / Screenshot Upload / Clipboard Ctrl+V)
// --------------------------------------------------------------------------
function openAiDeadlineModal() {
  state.deadlineAiPrompt = '';
  state.deadlineAiImage = null;
  state.deadlineAiLoading = false;
  state.deadlineAiStatus = '';
  state.extractedPreview = [];
  state.deadlineTab = 'prompt';
  renderAiModal();
}

function renderAiModal() {
  let modalEl = document.getElementById('deadline-ai-modal');
  if (!modalEl) {
    modalEl = document.createElement('div');
    modalEl.id = 'deadline-ai-modal';
    modalEl.className = 'overlay';
    document.body.appendChild(modalEl);
  }

  const prov = state.settings.provider || 'ollama';
  const model = state.settings.summaryModel || 'qwen2.5:7b';
  const providerLabel = prov === 'ollama' ? `AI Lokal (${model})` : `Online (${prov} · ${model})`;
  const isGcalConnected = !!state.gcalStatus.connected;

  modalEl.innerHTML = `<div class="modal ai-deadline-dialog" onclick="event.stopPropagation()">
    <div class="modal-head-row">
      <div class="dialog-title-group">
        <div class="spark-icon-circle">${icon('spark')}</div>
        <div>
          <h2>Ekstrak Deadline dengan AI</h2>
          <p>Otomatis susun tugas & tenggat dari prompt teks atau tangkapan layar (screenshot).</p>
        </div>
      </div>
      <button class="dismiss" data-action="close-ai-modal">×</button>
    </div>

    <!-- Direct Sync Banner (Privacy friendly: Green indicator without showing email) -->
    <div class="gcal-privacy-banner ${isGcalConnected ? 'connected' : 'disconnected'}">
      <span class="gcal-dot ${isGcalConnected ? 'green' : 'red'}"></span>
      <span>${isGcalConnected ? 'Google Calendar terhubung — Jadwal akan otomatis tersinkron.' : 'Google Calendar belum terhubung.'}</span>
      ${!isGcalConnected ? '<button class="button small ghost" style="padding:2px 8px;height:24px" data-action="open-gcal-connect-modal">Sambungkan</button>' : ''}
    </div>

    <div class="ai-input-tabs">
      <button class="ai-tab-btn ${state.deadlineTab === 'prompt' ? 'active' : ''}" data-action="switch-ai-tab" data-id="prompt">
        ${icon('notes')} Dari Prompt Teks
      </button>
      <button class="ai-tab-btn ${state.deadlineTab === 'screenshot' ? 'active' : ''}" data-action="switch-ai-tab" data-id="screenshot">
        ${icon('image')} Dari Screenshot / Gambar
      </button>
    </div>

    <div class="ai-dialog-body">
      ${state.deadlineTab === 'prompt' ? `
      <div class="prompt-tab-panel">
        <label class="field-label" for="ai-deadline-prompt">Tulis atau tempel teks tugas / jadwal:</label>
        <textarea id="ai-deadline-prompt" class="ai-prompt-area" placeholder="Contoh: 'Perkuliahan hari ini jam 16.30-19.29 link meet.google.com/xyz', atau 'Tugas resume bab 4 AI dikumpul 25 November jam 12 siang'…">${esc(state.deadlineAiPrompt)}</textarea>
        <div class="quick-prompt-chips">
          <span class="chip-label">Contoh cepat:</span>
          <button class="chip-btn" data-action="apply-prompt-chip" data-id="Perkuliahan Google Meet hari ini jam 16:30 link meet.google.com/bfd-ghon-ght">Kuliah Meet</button>
          <button class="chip-btn" data-action="apply-prompt-chip" data-id="Tugas makalah Kecerdasan Buatan dikumpul jumat depan jam 12:00, kuis Basis Data tanggal 25 jam 10:00">Tugas Kuliah</button>
          <button class="chip-btn" data-action="apply-prompt-chip" data-id="Laporan praktikum fisika deadline 3 hari lagi jam 23:59, revisi proposal senin depan">Praktikum</button>
        </div>
      </div>` : `
      <div class="screenshot-tab-panel">
        <div class="screenshot-dropzone ${state.deadlineAiImage ? 'has-image' : ''}" id="ai-screenshot-dropzone">
          ${state.deadlineAiImage ? `
            <div class="image-preview-card">
              <div class="image-preview-box">
                <img src="${state.deadlineAiImage}" alt="Screenshot tugas atau jadwal" class="screenshot-preview-img">
              </div>
              <div class="image-preview-controls">
                <button class="button small secondary" data-action="trigger-screenshot-upload">${icon('upload')} Ganti Gambar</button>
                <button class="button small ghost text-danger" data-action="remove-ai-image">${icon('trash')} Hapus</button>
              </div>
            </div>
          ` : `
            <div class="dropzone-hint" data-action="trigger-screenshot-upload">
              <div class="dropzone-icon">${icon('image')}</div>
              <strong>Pilih atau seret tangkapan layar jadwal ke sini</strong>
              <p>Dapat membaca chat WhatsApp, pengumuman kuliah, atau web jadwal</p>
              <span class="dropzone-types">Format: PNG, JPG, JPEG, WEBP · Tempel cepat dengan <b>Ctrl + V</b></span>
            </div>
          `}
        </div>
        <input type="file" id="ai-screenshot-file" accept="image/png,image/jpeg,image/webp" hidden style="display:none !important">
        <div class="ai-extra-field-group">
          <label class="field-label" for="ai-extra-prompt">Catatan / Instruksi tambahan (opsional):</label>
          <input type="text" id="ai-extra-prompt" class="textbox" placeholder="Tulis instruksi tambahan jika ada (opsional)…" value="${esc(state.deadlineAiPrompt)}" autocomplete="off" spellcheck="false">
        </div>
      </div>`}

      ${state.deadlineAiLoading ? `
      <div class="ai-processing-box" id="ai-processing-box">
        <div class="calm-spinner"></div>
        <strong>Sedang menganalisis jadwal…</strong>
        <p id="ai-status-text">${esc(state.deadlineAiStatus || 'Membaca dan menyusun jadwal tugas…')}</p>
      </div>` : ''}

      ${state.extractedPreview.length ? `
      <div class="extracted-results-container">
        <div class="space-between" style="margin-bottom:8px">
          <strong>Hasil Ekstraksi (${state.extractedPreview.length} tugas ditemukan):</strong>
          <span class="field-help">Periksa dan klik "Tambahkan ke Kalender"</span>
        </div>
        <div class="extracted-items-scroll">
          ${state.extractedPreview.map((item, idx) => `
            <div class="extracted-item-card">
              <div class="space-between">
                <input type="text" class="textbox item-edit-title" data-idx="${idx}" data-field="title" value="${esc(item.title)}" autocomplete="off" spellcheck="false">
                <span class="prio-badge prio-${item.priority || 'sedang'}">${esc(item.priority || 'sedang')}</span>
              </div>
              <div class="extracted-item-grid">
                <div><small>MATA KULIAH</small><input type="text" class="textbox small" data-idx="${idx}" data-field="course" value="${esc(item.course || 'Umum')}" autocomplete="off" spellcheck="false"></div>
                <div><small>TANGGAL</small><input type="date" class="textbox small" data-idx="${idx}" data-field="deadline" value="${esc(item.deadline)}"></div>
                <div><small>JAM</small><input type="time" class="textbox small" data-idx="${idx}" data-field="time" value="${esc(item.time || '23:59')}"></div>
              </div>
              ${item.description ? `<p class="extracted-item-desc">${esc(item.description)}</p>` : ''}
            </div>
          `).join('')}
        </div>
      </div>` : ''}
    </div>

    <div class="modal-footer-row">
      <div class="ai-provider-badge">
        ${icon(prov === 'ollama' ? 'cpu' : 'globe')} Model: <strong>${prov === 'ollama' ? 'AI Lokal' : 'AI Online'}</strong>
      </div>
      <div style="display:flex;gap:8px">
        <button class="button" data-action="close-ai-modal">Batal</button>
        ${state.extractedPreview.length ? `
          <button class="button accent" data-action="save-extracted-deadlines">
            ${icon('check')} Tambahkan Semua (${state.extractedPreview.length}) ke Kalender
          </button>
        ` : `
          <button class="button accent" data-action="run-deadline-extraction" ${state.deadlineAiLoading ? 'disabled' : ''}>
            ${icon('spark')} ${state.deadlineAiLoading ? 'Mengekstrak…' : 'Ekstrak Jadwal'}
          </button>
        `}
      </div>
    </div>
  </div>`;
}

// --------------------------------------------------------------------------
// MODAL: MANUAL ADD / EDIT DEADLINE
// --------------------------------------------------------------------------
function openManualDeadlineModal(editItem = null, defaultDate = null) {
  const isEdit = !!editItem;
  const tomorrow = new Date();
  tomorrow.setDate(tomorrow.getDate() + 1);
  const defDateStr = defaultDate || tomorrow.toISOString().slice(0, 10);

  const item = editItem || {
    id: '',
    title: '',
    course: '',
    deadline: defDateStr,
    time: '23:59',
    description: '',
    priority: 'sedang',
    status: 'pending'
  };

  let remVal = item.reminder !== undefined ? item.reminder : 30;
  let remAmount = 30;
  let remUnit = 'minutes';
  if (remVal === 0) {
    remAmount = 0;
    remUnit = 'none';
  } else if (remVal % 10080 === 0) {
    remAmount = remVal / 10080;
    remUnit = 'weeks';
  } else if (remVal % 1440 === 0) {
    remAmount = remVal / 1440;
    remUnit = 'days';
  } else if (remVal % 60 === 0) {
    remAmount = remVal / 60;
    remUnit = 'hours';
  } else {
    remAmount = remVal;
    remUnit = 'minutes';
  }

  const uid = Date.now();
  const content = `<div class="manual-deadline-form" style="display:flex;flex-direction:column;gap:14px;">
    <div class="field">
      <label class="field-label" for="manual-dl-title">Nama Tugas / Judul *</label>
      <input type="text" id="manual-dl-title" name="task_title_${uid}" class="textbox" placeholder="Judul tugas atau agenda perkuliahan…" value="${esc(item.title)}" autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" data-lpignore="true" data-1p-ignore="true" data-bwignore="true">
    </div>
    <div class="split-field">
      <div class="field">
        <label class="field-label" for="manual-dl-course">Mata Kuliah / Kategori</label>
        <input type="text" id="manual-dl-course" name="task_course_${uid}" class="textbox" placeholder="Nama mata kuliah atau kategori…" value="${esc(item.course)}" autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" data-lpignore="true" data-1p-ignore="true" data-bwignore="true">
      </div>
      <div class="field">
        <label class="field-label" for="manual-dl-prio">Prioritas</label>
        <select id="manual-dl-prio" class="control-select">
          <option value="tinggi" ${item.priority === 'tinggi' ? 'selected' : ''}>Tinggi (Mendesak / Bobot Besar)</option>
          <option value="sedang" ${item.priority === 'sedang' ? 'selected' : ''}>Sedang (Tugas Reguler)</option>
          <option value="rendah" ${item.priority === 'rendah' ? 'selected' : ''}>Rendah (Tugas Bacaan / Opsional)</option>
        </select>
      </div>
    </div>
    <div class="split-field">
      <div class="field">
        <label class="field-label" for="manual-dl-date">Tanggal Tenggat (Deadline) *</label>
        <input type="date" id="manual-dl-date" class="textbox" value="${esc(item.deadline)}" autocomplete="off">
      </div>
      <div class="field">
        <label class="field-label" for="manual-dl-time">Jam Pengumpulan</label>
        <input type="time" id="manual-dl-time" class="textbox" value="${esc(item.time || '23:59')}" autocomplete="off">
      </div>
    </div>
    <div class="field">
      <label class="field-label">Pengingat Notifikasi Kalender</label>
      <div style="display:flex;gap:8px;align-items:center;">
        <input type="number" id="manual-dl-reminder-num" class="textbox" min="1" max="99" value="${remAmount || 30}" style="max-width:90px" ${remUnit === 'none' ? 'disabled' : ''}>
        <select id="manual-dl-reminder-unit" class="control-select" style="flex:1" onchange="document.getElementById('manual-dl-reminder-num').disabled = (this.value === 'none');">
          <option value="minutes" ${remUnit === 'minutes' ? 'selected' : ''}>Menit sebelum tenggat</option>
          <option value="hours" ${remUnit === 'hours' ? 'selected' : ''}>Jam sebelum tenggat</option>
          <option value="days" ${remUnit === 'days' ? 'selected' : ''}>Hari sebelum tenggat</option>
          <option value="weeks" ${remUnit === 'weeks' ? 'selected' : ''}>Minggu sebelum tenggat</option>
          <option value="none" ${remUnit === 'none' ? 'selected' : ''}>Tanpa notifikasi</option>
        </select>
      </div>
    </div>
    <div class="field">
      <label class="field-label" for="manual-dl-desc">Deskripsi & Catatan Tambahan</label>
      <textarea id="manual-dl-desc" class="textbox" style="height:75px;resize:vertical" placeholder="Format file PDF, link pengumpulan, atau catatan penting..." autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false">${esc(item.description)}</textarea>
    </div>
    <label class="manual-form-gcal-sync">
      <input type="checkbox" id="manual-sync-gcal" ${state.gcalStatus.connected || state.gcalStatus.preferredMode === 'web' ? 'checked' : ''}>
      <span>${state.gcalStatus.connected ? 'Simpan & sinkronkan otomatis ke Google Calendar' : 'Simpan & buka jadwal di Google Calendar'}</span>
    </label>
  </div>`;

  modal(isEdit ? 'Edit Tenggat Tugas' : 'Tambah Tenggat Tugas Baru', content, [
    ['Batal', 'close-modal', 'ghost'],
    [isEdit ? 'Simpan Perubahan' : 'Tambah Tugas', 'save-manual-deadline', 'accent']
  ]);

  window.__manualDeadlineEditingId = item.id || null;
}

// --------------------------------------------------------------------------
// EVENT HANDLERS & INITIALIZATION
// --------------------------------------------------------------------------
const previousDeadlineClick = handleClick;
handleClick = function(action, id) {
  // Navigation & View switcher
  if (action === 'switch-deadline-view') {
    state.deadlineView = id;
    state.gcalMenuOpen = false;
    refreshPage();
    return;
  }
  if (action === 'filter-deadline') {
    state.deadlineFilter = id;
    state.gcalMenuOpen = false;
    refreshPage();
    return;
  }
  if (action === 'clear-deadline-search') {
    state.deadlineSearch = '';
    refreshPage();
    return;
  }

  // Google Calendar Modal & Actions
  if (action === 'open-gcal-connect-modal') {
    state.gcalMenuOpen = false;
    openGcalConnectModal(true);
    return;
  }
  if (action === 'start-gcal-auth-flow') {
    if (!state.gcalStatus.hasCredentials && !state.gcalStatus.clientId) {
      closeGcalModal();
      openGcalCredsModal();
    } else {
      startGcalAuth();
    }
    return;
  }
  if (action === 'open-gcal-creds-modal') {
    closeGcalModal();
    openGcalCredsModal();
    return;
  }
  if (action === 'close-gcal-creds-modal') {
    closeGcalCredsModal();
    return;
  }
  if (action === 'switch-creds-tab') {
    const cid = document.getElementById('gcal-input-client-id')?.value?.trim();
    const csec = document.getElementById('gcal-input-client-secret')?.value?.trim();
    if (cid !== undefined && cid !== '') state.gcalStatus.clientId = cid;
    if (csec !== undefined && csec !== '') state.gcalStatus.clientSecret = csec;
    state.gcalCredsTab = id;
    renderGcalCredsModal();
    return;
  }
  if (action === 'open-gcal-console-link') {
    openExternalUrl('https://console.cloud.google.com/apis/credentials');
    toast('Membuka Google Cloud Console di peramban…');
    return;
  }
  if (action === 'save-gcal-creds-and-auth') {
    saveGcalCredsAndAuth();
    return;
  }
  if (action === 'close-gcal-modal') {
    closeGcalModal();
    return;
  }
  if (action === 'choose-gcal-web-mode') {
    state.gcalStatus.preferredMode = 'web';
    updateGcalConfig({ preferredMode: 'web' });
    closeGcalModal();
    closeGcalCredsModal();
    closeGcalUserGuideModal();
    toast('✓ Mode Cepat Web aktif! Tenggat tugas otomatis membuka Google Calendar di browser dengan data terisi.');
    refreshPage();
    return;
  }
  if (action === 'open-gcal-user-guide') {
    closeGcalModal();
    openGcalUserGuideModal();
    return;
  }
  if (action === 'close-gcal-guide-modal') {
    closeGcalUserGuideModal();
    return;
  }
  if (action === 'open-gcal-web-test') {
    openExternalUrl('https://calendar.google.com');
    toast('Membuka Google Calendar di peramban…');
    return;
  }
  if (action === 'dismiss-gcal-modal') {
    const dontShow = document.getElementById('gcal-dont-show-again')?.checked;
    if (dontShow) {
      updateGcalConfig({ dismissPrompt: true });
    }
    closeGcalModal();
    return;
  }
  if (action === 'start-gcal-auth') {
    startGcalAuth();
    return;
  }
  if (action === 'cancel-gcal-auth') {
    if (window.__gcalPollTimer) {
      clearInterval(window.__gcalPollTimer);
      window.__gcalPollTimer = null;
    }
    state.gcalConnecting = false;
    renderGcalModal();
    return;
  }
  if (action === 'open-gcal-link') {
    let url = id || 'https://calendar.google.com';
    if (url.includes('action=TEMPLATE') || url.includes('/r/eventedit?text=')) {
      url = 'https://calendar.google.com';
    }
    openExternalUrl(url);
    return;
  }
  if (action === 'toggle-gcal-menu') {
    state.gcalMenuOpen = !state.gcalMenuOpen;
    refreshPage();
    return;
  }
  if (action === 'toggle-mask-email') {
    state.showGcalEmail = !state.showGcalEmail;
    refreshPage();
    return;
  }
  if (action === 'toggle-header-indicator') {
    state.settings.showGcalIndicator = !state.settings.showGcalIndicator;
    persistSettings();
    refreshPage();
    toast(state.settings.showGcalIndicator ? 'Indikator Google Calendar ditampilkan di header.' : 'Indikator Google Calendar disembunyikan dari header.');
    return;
  }
  if (action === 'go-gcal-settings') {
    state.gcalMenuOpen = false;
    state.page = 'settings';
    state.tab = 'general';
    refreshPage();
    return;
  }
  if (action === 'toggle-gcal-autosync') {
    const isNowAuto = !state.gcalStatus.autoSync;
    updateGcalConfig({ autoSync: isNowAuto });
    toast(isNowAuto ? 'Sinkronisasi otomatis Google Calendar diaktifkan.' : 'Sinkronisasi otomatis dinonaktifkan.');
    return;
  }
  if (action === 'save-gcal-client-id') {
    const val = document.getElementById('gcal-custom-client-id')?.value?.trim();
    updateGcalConfig({ clientId: val || '' });
    toast(val ? 'Client ID Google disimpan. Klik "Sambungkan dengan Akun Google" untuk otorisasi.' : 'Client ID Google dikosongkan (Mode Cepat aktif).');
    return;
  }
  if (action === 'gcal-disconnect') {
    disconnectGcal();
    return;
  }
  if (action === 'gcal-sync-all') {
    syncAllDeadlinesGcal();
    return;
  }
  if (action === 'sync-single-gcal') {
    const item = state.deadlines.find(d => d.id === id);
    if (item) syncSingleDeadlineGcal(item);
    return;
  }

  // Calendar Month Navigation
  if (action === 'cal-prev') {
    if (state.calendarMonth === 0) {
      state.calendarMonth = 11;
      state.calendarYear -= 1;
    } else {
      state.calendarMonth -= 1;
    }
    state.gcalMenuOpen = false;
    refreshPage();
    return;
  }
  if (action === 'cal-next') {
    if (state.calendarMonth === 11) {
      state.calendarMonth = 0;
      state.calendarYear += 1;
    } else {
      state.calendarMonth += 1;
    }
    state.gcalMenuOpen = false;
    refreshPage();
    return;
  }
  if (action === 'cal-today') {
    const now = new Date();
    state.calendarYear = now.getFullYear();
    state.calendarMonth = now.getMonth();
    state.gcalMenuOpen = false;
    refreshPage();
    return;
  }
  if (action === 'select-cal-date') {
    state.selectedDeadlineDate = id;
    state.gcalMenuOpen = false;
    refreshPage();
    return;
  }
  if (action === 'close-cal-details') {
    state.selectedDeadlineDate = null;
    refreshPage();
    return;
  }

  // Task Actions
  if (action === 'toggle-deadline') {
    promptCompleteAndRemoveDeadline(id);
    return;
  }
  if (action === 'confirm-complete-deadline') {
    const idToComplete = window.__deadlineIdToComplete;
    if (idToComplete) {
      completeAndRemoveDeadlineWithAnimation(idToComplete);
      window.__deadlineIdToComplete = null;
    }
    return;
  }
  if (action === 'cancel-complete-deadline') {
    window.__deadlineIdToComplete = null;
    closeModal();
    refreshPage();
    return;
  }
  if (action === 'delete-deadline') {
    promptDeleteDeadline(id);
    return;
  }
  if (action === 'confirm-delete-deadline') {
    const idToDelete = window.__deadlineIdToDelete;
    if (idToDelete) {
      removeDeadline(idToDelete);
      if (state.selectedDeadlineDate) {
        const remaining = state.deadlines.filter(d => d.deadline === state.selectedDeadlineDate);
        if (!remaining.length) state.selectedDeadlineDate = null;
      }
      toast('✓ Tugas berhasil dihapus.');
      window.__deadlineIdToDelete = null;
    }
    closeModal();
    refreshPage();
    return;
  }
  if (action === 'edit-deadline') {
    const item = state.deadlines.find(d => d.id === id);
    if (item) openManualDeadlineModal(item);
    return;
  }
  if (action === 'export-ics') {
    exportDeadlinesICS();
    return;
  }

  // AI Modal Actions
  if (action === 'open-deadline-ai') {
    state.gcalMenuOpen = false;
    openAiDeadlineModal();
    return;
  }
  if (action === 'close-ai-modal') {
    const el = document.getElementById('deadline-ai-modal');
    if (el) el.remove();
    return;
  }
  if (action === 'switch-ai-tab') {
    state.deadlineTab = id;
    renderAiModal();
    return;
  }
  if (action === 'apply-prompt-chip') {
    state.deadlineAiPrompt = id;
    renderAiModal();
    return;
  }
  if (action === 'trigger-screenshot-upload') {
    document.getElementById('ai-screenshot-file')?.click();
    return;
  }
  if (action === 'remove-ai-image') {
    state.deadlineAiImage = null;
    renderAiModal();
    return;
  }
  if (action === 'run-deadline-extraction') {
    const promptInput = document.getElementById('ai-deadline-prompt')?.value || document.getElementById('ai-extra-prompt')?.value || state.deadlineAiPrompt;
    state.deadlineAiPrompt = promptInput;
    runDeadlineAiExtraction();
    return;
  }
  if (action === 'save-extracted-deadlines') {
    // Read any edits from input fields
    document.querySelectorAll('.item-edit-title').forEach(el => {
      const idx = Number(el.dataset.idx);
      if (state.extractedPreview[idx]) state.extractedPreview[idx].title = el.value;
    });
    document.querySelectorAll('[data-field="course"]').forEach(el => {
      const idx = Number(el.dataset.idx);
      if (state.extractedPreview[idx]) state.extractedPreview[idx].course = el.value;
    });
    document.querySelectorAll('[data-field="deadline"]').forEach(el => {
      const idx = Number(el.dataset.idx);
      if (state.extractedPreview[idx]) state.extractedPreview[idx].deadline = el.value;
    });
    document.querySelectorAll('[data-field="time"]').forEach(el => {
      const idx = Number(el.dataset.idx);
      if (state.extractedPreview[idx]) state.extractedPreview[idx].time = el.value;
    });

    const defaultReminder = parseInt(state.settings.gcalReminder ?? 30, 10);
    const itemsToSave = state.extractedPreview.map(item => ({
      ...item,
      reminder: item.reminder !== undefined ? item.reminder : defaultReminder
    }));
    persistDeadlinesBatch(itemsToSave).then(async () => {
      const el = document.getElementById('deadline-ai-modal');
      if (el) el.remove();

      // If Google Calendar is connected, batch save already synced each new deadline
      if (state.gcalStatus.connected && state.gcalStatus.autoSync !== false) {
        toast(`✓ ${itemsToSave.length} tugas tersimpan & otomatis tersinkron ke Google Calendar!`);
        try {
          if (typeof api === 'function' && engine.ready) {
            await loadDeadlinesData();
          }
        } catch (_) {}
      } else {
        toast(`✓ Berhasil menambahkan ${itemsToSave.length} tugas ke kalender!`);
      }
      refreshPage();
    });
    return;
  }

  // Manual Modal Actions
  if (action === 'open-manual-deadline') {
    state.gcalMenuOpen = false;
    openManualDeadlineModal();
    return;
  }
  if (action === 'open-manual-deadline-date') {
    state.selectedDeadlineDate = null;
    openManualDeadlineModal(null, id);
    return;
  }
  if (action === 'save-manual-deadline') {
    const title = document.getElementById('manual-dl-title')?.value.trim();
    if (!title) {
      toast('Nama tugas tidak boleh kosong.');
      return;
    }
    const deadline = document.getElementById('manual-dl-date')?.value || new Date().toISOString().slice(0, 10);
    const remNum = parseInt(document.getElementById('manual-dl-reminder-num')?.value || '30', 10);
    const remUnit = document.getElementById('manual-dl-reminder-unit')?.value || 'minutes';
    let reminder = 30;
    if (remUnit === 'none') {
      reminder = 0;
    } else if (remUnit === 'minutes') {
      reminder = Math.max(1, isNaN(remNum) ? 30 : remNum);
    } else if (remUnit === 'hours') {
      reminder = Math.max(1, isNaN(remNum) ? 1 : remNum) * 60;
    } else if (remUnit === 'days') {
      reminder = Math.max(1, isNaN(remNum) ? 1 : remNum) * 1440;
    } else if (remUnit === 'weeks') {
      reminder = Math.max(1, isNaN(remNum) ? 1 : remNum) * 10080;
    }
    const item = {
      id: window.__manualDeadlineEditingId || undefined,
      title,
      course: document.getElementById('manual-dl-course')?.value.trim() || 'Umum',
      deadline,
      time: document.getElementById('manual-dl-time')?.value || '23:59',
      priority: document.getElementById('manual-dl-prio')?.value || 'sedang',
      description: document.getElementById('manual-dl-desc')?.value.trim() || '',
      reminder,
      status: 'pending'
    };
    const shouldSyncGcal = document.getElementById('manual-sync-gcal')?.checked;
    persistDeadline(item).then(savedItem => {
      closeModal();
      toast('Tenggat tugas berhasil disimpan.');
      refreshPage();
      if (shouldSyncGcal && savedItem && !state.gcalStatus.connected) {
        syncSingleDeadlineGcal(savedItem, true);
      }
    });
    return;
  }

  return previousDeadlineClick(action, id);
};

// Handle outside clicks to close dropdown menu
document.addEventListener('click', e => {
  if (state.gcalMenuOpen && !e.target.closest('.gcal-popover-wrap')) {
    state.gcalMenuOpen = false;
    refreshPage();
  }
});

// Handle file input for screenshot upload
document.addEventListener('change', e => {
  if (e.target.id === 'ai-screenshot-file') {
    const file = e.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = ev => {
        state.deadlineAiImage = ev.target.result;
        renderAiModal();
      };
      reader.readAsDataURL(file);
    }
  }
});

// Drag & drop support for screenshot dropzone
document.addEventListener('dragover', e => {
  const dropzone = e.target.closest('#ai-screenshot-dropzone');
  if (dropzone) {
    e.preventDefault();
    dropzone.classList.add('dragover');
  }
});

document.addEventListener('dragleave', e => {
  const dropzone = e.target.closest('#ai-screenshot-dropzone');
  if (dropzone && !dropzone.contains(e.relatedTarget)) {
    dropzone.classList.remove('dragover');
  }
});

document.addEventListener('drop', e => {
  const dropzone = e.target.closest('#ai-screenshot-dropzone');
  if (dropzone) {
    e.preventDefault();
    dropzone.classList.remove('dragover');
    const file = e.dataTransfer?.files?.[0];
    if (file && file.type.startsWith('image/')) {
      const reader = new FileReader();
      reader.onload = ev => {
        state.deadlineAiImage = ev.target.result;
        state.deadlineTab = 'screenshot';
        renderAiModal();
      };
      reader.readAsDataURL(file);
    }
  }
});

// Handle search input live
document.addEventListener('input', e => {
  if (e.target.id === 'deadline-search-input') {
    state.deadlineSearch = e.target.value;
    const container = document.querySelector('.deadlines-body');
    if (container) {
      const all = state.deadlines || [];
      const q = (state.deadlineSearch || '').toLowerCase().trim();
      const filtered = all.filter(d => !q || (d.title + ' ' + (d.course || '') + ' ' + (d.description || '')).toLowerCase().includes(q));
      container.innerHTML = state.deadlineView === 'calendar' ? renderDeadlineCalendar() : renderDeadlineList(filtered);
    }
  }
  if (e.target.id === 'ai-deadline-prompt' || e.target.id === 'ai-extra-prompt') {
    state.deadlineAiPrompt = e.target.value;
  }
});

// Clipboard listener: Paste screenshot (Ctrl+V) anywhere when AI modal is open!
document.addEventListener('paste', e => {
  const modalEl = document.getElementById('deadline-ai-modal');
  if (!modalEl) return;
  const items = e.clipboardData?.items;
  if (!items) return;
  for (const item of items) {
    if (item.type.startsWith('image/')) {
      const file = item.getAsFile();
      if (file) {
        const reader = new FileReader();
        reader.onload = ev => {
          state.deadlineAiImage = ev.target.result;
          state.deadlineTab = 'screenshot';
          renderAiModal();
          toast('Screenshot dari clipboard berhasil ditempel!');
        };
        reader.readAsDataURL(file);
        break;
      }
    }
  }
});

// Run AI extraction
async function runDeadlineAiExtraction() {
  state.deadlineAiLoading = true;
  state.deadlineAiStatus = 'Menghubungkan ke layanan AI…';
  state.extractedPreview = [];
  renderAiModal();

  try {
    const payload = {
      prompt: state.deadlineAiPrompt || '',
      image: state.deadlineAiImage || '',
      provider: state.settings.provider || 'ollama',
      model: state.settings.summaryModel || 'qwen2.5:7b',
      apiKey: state.apiKey || '',
      endpoint: state.settings.endpoint || ''
    };

    if (!payload.prompt.trim() && !payload.image) {
      throw Error('Masukkan prompt teks atau pilih gambar screenshot terlebih dahulu.');
    }

    if (typeof api === 'function' && engine.ready) {
      const jobRes = await api('/deadlines/extract', payload);
      const jid = jobRes.job;
      let finished = false;
      while (!finished) {
        await new Promise(r => setTimeout(r, 450));
        const st = await api('/status');
        const job = st.jobs?.[jid];
        if (!job) break;
        if (job.status === 'running') {
          state.deadlineAiStatus = job.message || 'Menganalisis jadwal tugas…';
          const p = document.getElementById('ai-status-text');
          if (p) p.textContent = state.deadlineAiStatus;
        } else if (job.status === 'done') {
          finished = true;
          const result = job.result || {};
          state.extractedPreview = result.deadlines || [];
          if (!state.extractedPreview.length) {
            toast('AI tidak menemukan tenggat tugas dalam input yang diberikan.');
          }
        } else if (job.status === 'error' || job.status === 'cancelled') {
          throw Error(job.message || 'Gagal mengekstrak deadline.');
        }
      }
    } else {
      // Offline simulation fallback for prototyping
      await new Promise(r => setTimeout(r, 1200));
      const tomorrow = new Date();
      tomorrow.setDate(tomorrow.getDate() + 3);
      state.extractedPreview = [
        {
          id: 'dl_sample_' + Date.now(),
          title: 'Tugas Resume Bab 4 AI',
          course: 'Kecerdasan Buatan',
          deadline: tomorrow.toISOString().slice(0, 10),
          time: '12:00',
          description: 'Ekstraksi otomatis dari prompt',
          priority: 'tinggi'
        }
      ];
    }
  } catch (err) {
    alertError(err);
  } finally {
    state.deadlineAiLoading = false;
    renderAiModal();
  }
}

// Initial loads
loadDeadlinesData();
loadGcalStatus();
startGcalBackgroundPolling();
