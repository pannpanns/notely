/* Notely Deadline Notes: Calendar view, list view, privacy-first Google Calendar integration, and AI extraction from text & screenshots. */
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

// Google Calendar Integration State (Privacy First)
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
state.gcalAdvancedOpen = false;
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

async function loadGcalStatus() {
  try {
    if (typeof api === 'function' && engine.ready) {
      const data = await api('/gcalendar/status');
      if (data && typeof data.connected === 'boolean') {
        state.gcalStatus = data;
        safeStore.set('notelyGcalStatus', data);
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
  state.gcalAdvancedOpen = false;
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

function renderGcalModal() {
  let modalEl = document.getElementById('gcal-connect-modal');
  if (!modalEl) {
    modalEl = document.createElement('div');
    modalEl.id = 'gcal-connect-modal';
    modalEl.className = 'overlay';
    document.body.appendChild(modalEl);
  }

  const isConnected = state.gcalStatus.connected;
  const clientId = state.gcalStatus.clientId || '';

  modalEl.innerHTML = `<div class="gcal-modal-card" onclick="event.stopPropagation()">
    <div class="space-between" style="align-items:flex-start">
      <div class="gcal-logo-circle">${icon('google')}</div>
      <button class="dismiss" data-action="close-gcal-modal" title="Tutup">×</button>
    </div>

    ${isConnected ? `
      <!-- Connected State (Privacy friendly, no bare email leak) -->
      <h2 class="gcal-modal-title">✓ Google Calendar Terhubung!</h2>
      <p class="gcal-modal-subtitle">Akun Anda kini aktif tersambung secara privat dengan Notely.</p>

      <div class="gcal-success-box">
        <div class="gcal-success-icon">✓</div>
        <strong>Sinkronisasi Otomatis Aktif</strong>
        <p>Setiap tugas baru dari prompt teks atau screenshot pengumuman akan langsung otomatis masuk ke Google Calendar Anda tanpa perlu membuka browser lagi.</p>
      </div>

      <div class="gcal-modal-footer" style="justify-content:flex-end">
        <button class="button accent" data-action="close-gcal-modal">Mulai Gunakan</button>
      </div>
    ` : state.gcalConnecting ? `
      <!-- Waiting / Authorizing State -->
      <h2 class="gcal-modal-title">Menghubungkan Akun Google…</h2>
      <p class="gcal-modal-subtitle">Laman otorisasi resmi Google telah dibuka di peramban web Anda.</p>

      <div class="gcal-waiting-box">
        <div class="gcal-spinner"></div>
        <strong>Menunggu Otorisasi di Browser</strong>
        <p>Silakan klik <strong>"Lanjutkan"</strong> atau <strong>"Izinkan"</strong> pada tab Google yang terbuka. Halaman tersebut akan tertutup otomatis begitu berhasil!</p>
      </div>

      <div class="gcal-modal-footer" style="justify-content:space-between">
        <span class="field-help">Notely mendeteksi sambungan secara otomatis…</span>
        <button class="button danger" data-action="cancel-gcal-auth">Batalkan</button>
      </div>
    ` : `
      <!-- Initial Connect Prompt State -->
      <h2 class="gcal-modal-title">Sambungkan ke Google Calendar</h2>
      <p class="gcal-modal-subtitle">Otomatis sinkronkan tenggat tugas Anda langsung ke Google Calendar tanpa perlu membuka browser satu per satu.</p>

      <div class="gcal-benefits-list">
        <div class="gcal-benefit-item">
          <span class="benefit-icon">${icon('spark')}</span>
          <div><strong>Otomatis dari Screenshot & Prompt:</strong> AI mengekstrak tugas dan langsung menjadwalkannya di Google Calendar Anda.</div>
        </div>
        <div class="gcal-benefit-item">
          <span class="benefit-icon">${icon('bolt')}</span>
          <div><strong>Pengingat H-1 & Jam Deadline:</strong> Dapatkan notifikasi pengingat otomatis di ponsel dan komputer Anda.</div>
        </div>
        <div class="gcal-benefit-item">
          <span class="benefit-icon">${icon('check')}</span>
          <div><strong>Privasi Terjaga & Bebas Repot:</strong> 1-klik otorisasi Google via loopback lokal, aman dan tanpa mempublikasikan informasi akun Anda.</div>
        </div>
      </div>

      <div style="margin-bottom:14px">
        <button class="button accent" style="width:100%;height:44px;font-size:14px;gap:10px" data-action="start-gcal-auth">
          ${icon('google')} Sambungkan dengan Akun Google
        </button>
      </div>

      <!-- Advanced / Custom Client ID Accordion -->
      <div class="gcal-advanced-details">
        <button class="gcal-advanced-toggle" data-action="toggle-gcal-advanced">
          ${icon('settings')} Pengaturan Kredensial Pribadi (Opsional) ${icon('chevron')}
        </button>

        ${state.gcalAdvancedOpen ? `
          <div class="gcal-advanced-form">
            <p class="field-help" style="margin-bottom:6px">Gunakan Google Cloud Client ID milik Anda sendiri atau unggah file <code>credentials.json</code>:</p>
            <div class="field">
              <label class="field-label" for="gcal-client-id">Google Client ID</label>
              <input type="text" id="gcal-client-id" class="textbox" placeholder="xxxx.apps.googleusercontent.com" value="${esc(clientId)}">
            </div>
            <div class="field">
              <label class="field-label" for="gcal-client-sec">Google Client Secret</label>
              <input type="password" id="gcal-client-sec" class="textbox" placeholder="GOCSPX-xxxx">
            </div>
            <div class="gcal-drop-json" data-action="trigger-credentials-upload">
              ${icon('file')} Atau klik untuk unggah file <code>credentials.json</code> dari Google Console
            </div>
            <input type="file" id="gcal-creds-file" accept=".json" style="display:none">
          </div>
        ` : ''}
      </div>

      <div class="gcal-modal-footer">
        <label class="gcal-dismiss-check">
          <input type="checkbox" id="gcal-dont-show-again">
          <span>Jangan ingatkan lagi</span>
        </label>
        <div style="display:flex;gap:8px">
          <button class="button ghost" data-action="dismiss-gcal-modal">Nanti saja</button>
          <button class="button accent" data-action="start-gcal-auth">Sambungkan</button>
        </div>
      </div>
    `}
  </div>`;
}

// Start OAuth loopback authentication
async function startGcalAuth() {
  const customIdInput = document.getElementById('gcal-client-id');
  const customSecInput = document.getElementById('gcal-client-sec');
  const clientId = customIdInput ? customIdInput.value.trim() : '';
  const clientSecret = customSecInput ? customSecInput.value.trim() : '';

  state.gcalConnecting = true;
  renderGcalModal();

  try {
    if (typeof api === 'function' && engine.ready) {
      const res = await api('/gcalendar/auth-url', { clientId, clientSecret });
      if (res.authUrl) {
        window.open(res.authUrl, '_blank');

        // Start polling status
        let pollCount = 0;
        if (window.__gcalPollTimer) clearInterval(window.__gcalPollTimer);
        window.__gcalPollTimer = setInterval(async () => {
          pollCount++;
          if (pollCount > 80 || !state.gcalConnecting) {
            clearInterval(window.__gcalPollTimer);
            window.__gcalPollTimer = null;
            state.gcalConnecting = false;
            renderGcalModal();
            return;
          }
          try {
            const st = await api('/gcalendar/status');
            if (st && st.connected) {
              clearInterval(window.__gcalPollTimer);
              window.__gcalPollTimer = null;
              state.gcalStatus = st;
              state.gcalConnecting = false;
              safeStore.set('notelyGcalStatus', st);
              renderGcalModal();
              refreshPage();
              toast('✓ Google Calendar berhasil tersambung!');
            }
          } catch (e) {}
        }, 1500);
      }
    } else {
      // Prototype simulation fallback
      await new Promise(r => setTimeout(r, 1800));
      state.gcalStatus = {
        connected: true,
        email: 'user.mahasiswa@gmail.com',
        autoSync: true,
        hasCredentials: true,
        dismissPrompt: false
      };
      state.gcalConnecting = false;
      renderGcalModal();
      refreshPage();
      toast('✓ Google Calendar tersambung (Mode Demo)!');
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
      await api('/gcalendar/disconnect');
    }
    state.gcalStatus.connected = false;
    state.gcalStatus.email = '';
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

// Sync single event directly to Google Calendar via API
async function syncSingleDeadlineGcal(item) {
  if (!state.gcalStatus.connected) {
    openGcalConnectModal(true);
    return;
  }

  try {
    toast(`Menyinkronkan "${item.title}" ke Google Calendar…`);
    if (typeof api === 'function' && engine.ready) {
      const res = await api('/gcalendar/sync-event', { id: item.id, item });
      if (res.item) {
        const idx = state.deadlines.findIndex(d => d.id === item.id);
        if (idx !== -1) state.deadlines[idx] = res.item;
        safeStore.set('notelyDeadlines', state.deadlines);
        refreshPage();
      }
    } else {
      item.gcal_event_id = 'demo_event_' + Date.now();
      item.gcal_link = 'https://calendar.google.com';
      item.gcal_synced_at = new Date().toISOString();
      const idx = state.deadlines.findIndex(d => d.id === item.id);
      if (idx !== -1) state.deadlines[idx] = item;
      safeStore.set('notelyDeadlines', state.deadlines);
      refreshPage();
    }
    toast(`✓ "${item.title}" berhasil disinkronkan ke Google Calendar!`);
  } catch (err) {
    alertError(err);
  }
}

// Sync all active deadlines directly to Google Calendar via API
async function syncAllDeadlinesGcal() {
  if (!state.gcalStatus.connected) {
    openGcalConnectModal(true);
    return;
  }

  try {
    toast('Menyinkronkan semua tugas aktif ke Google Calendar…');
    if (typeof api === 'function' && engine.ready) {
      const res = await api('/gcalendar/sync-all');
      await loadDeadlinesData();
      toast(`✓ Berhasil menyinkronkan ${res.synced || 0} tugas ke Google Calendar!`);
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

    // If Google Calendar is connected and autoSync is enabled, sync event immediately!
    if (state.gcalStatus.connected && state.gcalStatus.autoSync !== false) {
      syncSingleDeadlineGcal(item);
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
  try {
    if (typeof api === 'function' && engine.ready) {
      await api('/deadlines/delete', { id });
    }
  } catch (e) {}
  state.deadlines = state.deadlines.filter(d => d.id !== id);
  safeStore.set('notelyDeadlines', state.deadlines);
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

  return `<div class="deadlines-page">
    <div class="deadlines-top-bar">
      <div class="deadlines-title-area">
        <span class="eyebrow">AGENDA & MANAJEMEN TUGAS</span>
        <h1>Tenggat Tugas</h1>
        <p>${countActive} tugas aktif · ${countUrgent ? `<strong class="urgent-text">${countUrgent} mendesak</strong>` : 'Semua terkendali'}</p>
      </div>

      <div class="deadlines-actions">
        <!-- Privacy-focused Google Calendar Indicator: Green (Connected) or Red (Disconnected) without exposing email -->
        ${isGcalConnected ? `
          <div class="gcal-popover-wrap">
            <button class="gcal-status-pill connected" data-action="toggle-gcal-menu" title="Google Calendar: Tersambung (Klik untuk opsi)">
              <span class="gcal-dot green"></span>
              <span>Google Calendar</span>
              ${icon('chevron')}
            </button>
            ${state.gcalMenuOpen ? renderGcalDropdownMenu() : ''}
          </div>
        ` : `
          <button class="gcal-status-pill disconnected" data-action="open-gcal-connect-modal" title="Google Calendar: Belum tersambung (Klik untuk menyambungkan)">
            <span class="gcal-dot red"></span>
            <span>Google Calendar</span>
          </button>
        `}

        <button class="button accent" data-action="open-deadline-ai">${icon('spark')} Buat dengan AI</button>
        <button class="button" data-action="open-manual-deadline">${icon('plus')} Tambah manual</button>
        <button class="button ghost" data-action="export-ics" title="Ekspor semua tugas aktif ke format .ics">${icon('google')} Ekspor .ics</button>
      </div>
    </div>

    <div class="deadlines-toolbar">
      <div class="segmented" role="tablist" aria-label="Mode Tampilan">
        <button class="${state.deadlineView === 'calendar' ? 'active' : ''}" data-action="switch-deadline-view" data-id="calendar">${icon('calendar')} Kalender</button>
        <button class="${state.deadlineView === 'list' ? 'active' : ''}" data-action="switch-deadline-view" data-id="list">${icon('notes')} Daftar Tugas</button>
      </div>

      <div class="deadlines-search-wrap">
        ${icon('search')}
        <input type="text" id="deadline-search-input" placeholder="Cari nama tugas atau mata kuliah…" value="${esc(state.deadlineSearch)}" autocomplete="off">
        ${state.deadlineSearch ? `<button class="search-clear" data-action="clear-deadline-search">×</button>` : ''}
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
  const isAutoSync = state.gcalStatus.autoSync !== false;
  const rawEmail = state.gcalStatus.email || '';
  const displayEmail = state.showGcalEmail ? rawEmail : maskEmail(rawEmail);

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
      <small>AKUN GOOGLE TERHUBUNG</small>
      <strong>${esc(displayEmail || 'Google Account')}</strong>
    </div>

    <div class="gcal-menu-row">
      <span>Sinkronkan tugas otomatis</span>
      <input type="checkbox" class="switch" ${isAutoSync ? 'checked' : ''} data-action="toggle-gcal-autosync" aria-label="Auto sync">
    </div>

    <div class="gcal-menu-actions">
      <button class="button small" data-action="gcal-sync-all">
        ${icon('bolt')} Sinkronkan Semua Tugas Sekarang
      </button>
      <button class="button small ghost" data-action="go-gcal-settings">
        ${icon('settings')} Pengaturan Akun di Settings
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
  const displayEmail = state.showGcalEmail ? rawEmail : maskEmail(rawEmail);

  return `<div class="setting-card">
    <div class="space-between" style="align-items:flex-start;margin-bottom:12px">
      <div>
        <h3 class="section-title">Integrasi Google Calendar (Tenggat Tugas)</h3>
        <p class="section-description" style="margin-bottom:0">Sinkronkan tenggat tugas dari prompt atau screenshot langsung ke Google Calendar.</p>
      </div>
      <div class="gcal-status-badge-inline ${isConnected ? 'connected' : 'disconnected'}">
        <span class="gcal-dot ${isConnected ? 'green' : 'red'}"></span>
        <strong>${isConnected ? 'Tersambung' : 'Belum Tersambung'}</strong>
      </div>
    </div>

    ${isConnected ? `
      <div>
        <div class="space-between" style="padding:12px 14px;background:#f7faf8;border-radius:9px;border:1px solid #e1ebe4;margin-bottom:14px">
          <div>
            <small style="font-size:11px;color:#6b8073;font-weight:700;text-transform:uppercase">Akun Google Terhubung</small>
            <div style="font-size:13.5px;font-weight:600;color:#1e352b;display:flex;align-items:center;gap:8px;margin-top:2px">
              <span>${esc(displayEmail || 'Akun Google Terotorisasi')}</span>
              ${rawEmail ? `
                <button class="button small ghost icon-only" data-action="toggle-mask-email" title="${state.showGcalEmail ? 'Sembunyikan email' : 'Tampilkan email'}" style="width:28px;height:28px">
                  ${icon(state.showGcalEmail ? 'eyeoff' : 'eye')}
                </button>
              ` : ''}
            </div>
          </div>
          <button class="button small ghost danger-hover" style="color:var(--danger)" data-action="gcal-disconnect">
            ${icon('trash')} Putuskan Akun
          </button>
        </div>

        <div class="setting-row" style="margin-bottom:14px">
          <div>
            <h4 style="font-size:13px;font-weight:600;color:#22362b;margin:0 0 4px">Sinkronisasi Otomatis Tugas Baru</h4>
            <p style="font-size:12px;color:#718579;margin:0">Setiap tugas yang diekstrak AI atau dibuat manual langsung dikirim ke Google Calendar di latar belakang.</p>
          </div>
          <input type="checkbox" class="switch" ${isAutoSync ? 'checked' : ''} data-action="toggle-gcal-autosync" aria-label="Auto sync">
        </div>

        <div style="display:flex;gap:10px;padding-top:10px;border-top:1px solid #eef3ef">
          <button class="button small" data-action="gcal-sync-all">${icon('bolt')} Sinkronkan Semua Tugas Sekarang</button>
          <button class="button small ghost" data-action="page" data-id="deadlines">${icon('calendar')} Buka Tenggat Tugas</button>
        </div>
      </div>
    ` : `
      <div>
        <p style="font-size:12.5px;color:#677d70;line-height:1.6;margin-bottom:14px">
          Hubungkan akun Google Calendar Anda agar tugas kuliah, proyek, dan deadline dapat otomatis tersinkron lengkap dengan notifikasi pengingat ke ponsel & laptop Anda.
        </p>
        <div style="display:flex;gap:10px;align-items:center;flex-wrap:wrap">
          <button class="button accent small" data-action="open-gcal-connect-modal">
            ${icon('google')} Sambungkan Google Calendar
          </button>
          <button class="button small ghost" data-action="open-gcal-connect-modal">
            ${icon('settings')} Pengaturan Kredensial Pribadi (Opsional)
          </button>
        </div>
      </div>
    `}
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

  return `<div class="calendar-wrapper">
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

    <div class="calendar-grid">
      <div class="calendar-day-header">Sen</div>
      <div class="calendar-day-header">Sel</div>
      <div class="calendar-day-header">Rab</div>
      <div class="calendar-day-header">Kam</div>
      <div class="calendar-day-header">Jum</div>
      <div class="calendar-day-header weekend">Sab</div>
      <div class="calendar-day-header weekend">Min</div>

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
// LIST VIEW: renderDeadlineList(tasks)
// --------------------------------------------------------------------------
function renderDeadlineList(tasks) {
  if (!tasks.length) {
    return `<div class="quiet-empty deadline-empty">
      <div class="empty-icon">${icon('calendar')}</div>
      <h3>Tidak ada tugas yang cocok</h3>
      <p>Gunakan tombol "Buat dengan AI" untuk mengekstrak tugas dari prompt atau screenshot pengumuman, atau tambahkan tugas baru secara manual.</p>
      <div style="display:flex;gap:10px;margin-top:14px;">
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
            <a href="${esc(d.gcal_link || 'https://calendar.google.com')}" target="_blank" class="button small ghost" title="Buka di Google Calendar">${icon('google')} Buka di Kalender</a>
          ` : isGcalConnected ? `
            <button class="button small accent-soft" data-action="sync-single-gcal" data-id="${esc(d.id)}" title="Kirim langsung ke Google Calendar di latar belakang">
              ${icon('google')} Sync Kalender
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
              ${!hasGcal ? `
                <button class="button small accent-soft icon-only" data-action="sync-single-gcal" data-id="${esc(t.id)}" title="Sinkronkan ke Google Calendar">${icon('google')}</button>
              ` : ''}
              <button class="button small ghost icon-only" data-action="delete-deadline" data-id="${esc(t.id)}" title="Hapus">${icon('trash')}</button>
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
      <span>${isGcalConnected ? 'Google Calendar aktif — Tugas akan langsung disinkronkan otomatis tanpa buka browser.' : 'Google Calendar belum tersambung.'}</span>
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
        <label class="field-label" for="ai-deadline-prompt">Tulis atau tempel teks tugas / pengumuman:</label>
        <textarea id="ai-deadline-prompt" class="ai-prompt-area" placeholder="Contoh: 'Tugas resume bab 4 AI dikumpul 25 November jam 12 siang, Tugas Proyek Web tanggal 28 jam 23:59, dan kuis basis data senin depan'...">${esc(state.deadlineAiPrompt)}</textarea>
        <div class="quick-prompt-chips">
          <span class="chip-label">Contoh cepat:</span>
          <button class="chip-btn" data-action="apply-prompt-chip" data-id="Tugas makalah Kecerdasan Buatan dikumpul jumat depan jam 12:00, kuis Basis Data tanggal 25 jam 10:00">Tugas Kuliah</button>
          <button class="chip-btn" data-action="apply-prompt-chip" data-id="Laporan praktikum fisika deadline 3 hari lagi jam 23:59, revisi proposal senin depan">Praktikum</button>
          <button class="chip-btn" data-action="apply-prompt-chip" data-id="Presentasi Final Project tanggal 30 November jam 13:00 materi bab 1-5">Presentasi</button>
        </div>
      </div>` : `
      <div class="screenshot-tab-panel">
        <div class="screenshot-dropzone ${state.deadlineAiImage ? 'has-image' : ''}" id="ai-screenshot-dropzone" data-action="trigger-screenshot-upload">
          ${state.deadlineAiImage ? `
            <div class="image-preview-wrapper">
              <img src="${state.deadlineAiImage}" alt="Screenshot tugas" class="screenshot-preview-img">
              <button class="button small danger remove-img-btn" data-action="remove-ai-image">${icon('trash')} Ganti Gambar</button>
            </div>
          ` : `
            <div class="dropzone-hint">
              <div class="dropzone-icon">${icon('image')}</div>
              <strong>Klik untuk memilih screenshot, atau drag & drop di sini</strong>
              <p>Bisa juga langsung <strong>tekan Ctrl+V</strong> untuk menempelkan hasil screenshot clipboard!</p>
              <span class="dropzone-types">Format: PNG, JPG, WEBP (Didukung OCR Windows Bawaan Instan)</span>
            </div>
          `}
        </div>
        <input type="file" id="ai-screenshot-file" accept="image/png,image/jpeg,image/webp" style="display:none">
        <div style="margin-top:12px">
          <label class="field-label" for="ai-extra-prompt">Catatan / Instruksi tambahan (opsional):</label>
          <input type="text" id="ai-extra-prompt" class="textbox" placeholder="Misal: 'Ambil hanya tugas minggu ini' atau 'Tandai prioritas tinggi'..." value="${esc(state.deadlineAiPrompt)}">
        </div>
      </div>`}

      ${state.deadlineAiLoading ? `
      <div class="ai-processing-box">
        <span class="working-orbit">${icon('spark')}</span>
        <strong>Sedang memproses dengan ${providerLabel}…</strong>
        <p>${esc(state.deadlineAiStatus || 'Membaca data dan menyusun jadwal tugas…')}</p>
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
                <input type="text" class="textbox item-edit-title" data-idx="${idx}" data-field="title" value="${esc(item.title)}">
                <span class="prio-badge prio-${item.priority || 'sedang'}">${esc(item.priority || 'sedang')}</span>
              </div>
              <div class="extracted-item-grid">
                <div><small>MATA KULIAH</small><input type="text" class="textbox small" data-idx="${idx}" data-field="course" value="${esc(item.course || 'Umum')}"></div>
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
        ${icon('cpu')} Diproses: <strong>${providerLabel}</strong>
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

  const content = `<div class="manual-deadline-form" style="display:flex;flex-direction:column;gap:14px;">
    <div class="field">
      <label class="field-label" for="manual-dl-title">Nama Tugas / Judul *</label>
      <input type="text" id="manual-dl-title" class="textbox" placeholder="Misal: Tugas Akhir PBO, Laporan Praktikum" value="${esc(item.title)}">
    </div>
    <div class="split-field">
      <div class="field">
        <label class="field-label" for="manual-dl-course">Mata Kuliah / Kategori</label>
        <input type="text" id="manual-dl-course" class="textbox" placeholder="Misal: PBO, Basis Data" value="${esc(item.course)}">
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
        <input type="date" id="manual-dl-date" class="textbox" value="${esc(item.deadline)}">
      </div>
      <div class="field">
        <label class="field-label" for="manual-dl-time">Jam Pengumpulan</label>
        <input type="time" id="manual-dl-time" class="textbox" value="${esc(item.time || '23:59')}">
      </div>
    </div>
    <div class="field">
      <label class="field-label" for="manual-dl-desc">Deskripsi & Catatan Tambahan</label>
      <textarea id="manual-dl-desc" class="textbox" style="height:75px;resize:vertical" placeholder="Format file PDF, link pengumpulan, atau catatan penting...">${esc(item.description)}</textarea>
    </div>
  </div>`;

  modal(isEdit ? 'Edit Tenggat Tugas' : 'Tambah Tenggat Tugas Baru', content, [
    ['Batal', 'close-modal', ''],
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
  if (action === 'close-gcal-modal') {
    closeGcalModal();
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
  if (action === 'toggle-gcal-advanced') {
    state.gcalAdvancedOpen = !state.gcalAdvancedOpen;
    renderGcalModal();
    return;
  }
  if (action === 'trigger-credentials-upload') {
    document.getElementById('gcal-creds-file')?.click();
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
    toggleDeadline(id);
    return;
  }
  if (action === 'delete-deadline') {
    removeDeadline(id);
    if (state.selectedDeadlineDate) {
      const remaining = state.deadlines.filter(d => d.deadline === state.selectedDeadlineDate);
      if (!remaining.length) state.selectedDeadlineDate = null;
    }
    toast('Tugas dihapus.');
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

    const itemsToSave = [...state.extractedPreview];
    persistDeadlinesBatch(itemsToSave).then(async () => {
      const el = document.getElementById('deadline-ai-modal');
      if (el) el.remove();

      // If Google Calendar is connected, automatically sync all new deadlines!
      if (state.gcalStatus.connected && state.gcalStatus.autoSync !== false) {
        toast(`Menyimpan ${itemsToSave.length} tugas & menyinkronkan langsung ke Google Calendar…`);
        try {
          if (typeof api === 'function' && engine.ready) {
            await api('/gcalendar/sync-all');
            await loadDeadlinesData();
          }
          toast(`✓ ${itemsToSave.length} tugas tersimpan & otomatis tersinkron ke Google Calendar!`);
        } catch (syncErr) {
          toast(`Tugas tersimpan di Notely. Sinkronisasi Google Calendar: ${syncErr.message || syncErr}`);
        }
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
    const item = {
      id: window.__manualDeadlineEditingId || undefined,
      title,
      course: document.getElementById('manual-dl-course')?.value.trim() || 'Umum',
      deadline,
      time: document.getElementById('manual-dl-time')?.value || '23:59',
      priority: document.getElementById('manual-dl-prio')?.value || 'sedang',
      description: document.getElementById('manual-dl-desc')?.value.trim() || '',
      status: 'pending'
    };
    persistDeadline(item).then(() => {
      closeModal();
      toast('Tenggat tugas berhasil disimpan.');
      refreshPage();
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

// Handle credentials.json upload in advanced section
document.addEventListener('change', e => {
  if (e.target.id === 'gcal-creds-file') {
    const file = e.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = ev => {
        try {
          const parsed = JSON.parse(ev.target.result);
          const creds = parsed.installed || parsed.web || parsed;
          const cId = creds.client_id || '';
          const cSec = creds.client_secret || '';
          if (cId) {
            const idInput = document.getElementById('gcal-client-id');
            const secInput = document.getElementById('gcal-client-sec');
            if (idInput) idInput.value = cId;
            if (secInput) secInput.value = cSec;
            toast('✓ Berhasil memuat Client ID dari file credentials.json!');
          } else {
            toast('Format credentials.json tidak memiliki client_id.');
          }
        } catch (err) {
          toast('Gagal membaca file credentials.json: file JSON tidak valid.');
        }
      };
      reader.readAsText(file);
    }
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
          renderAiModal();
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
