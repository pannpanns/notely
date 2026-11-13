/* Native updater; release notes are always treated as text, never HTML. */
(() => {
  const tauri = window.__TAURI__;
  if (!tauri?.core?.invoke) return;
  let checking = false;
  const busy = () => typeof state !== 'undefined' && (state.recording || state.summaryLoading) ||
    typeof engine !== 'undefined' && (engine.state === 'stopping' || engine.transcribeJob || engine.currentJob || engine.modelJob || engine.ollamaJob || engine.cudaJob);

  function formatBytes(bytes) {
    if (!bytes || isNaN(bytes)) return '0 B';
    if (bytes >= 1073741824) return (bytes / 1073741824).toFixed(1) + ' GB';
    if (bytes >= 1048576) return (bytes / 1048576).toFixed(1) + ' MB';
    if (bytes >= 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return bytes + ' B';
  }

  function formatSpeed(bps) {
    if (!bps || bps <= 0) return '';
    if (bps >= 1048576) return (bps / 1048576).toFixed(1) + ' MB/s';
    return (bps / 1024).toFixed(0) + ' KB/s';
  }

  function renderReleaseNotes(notesText, container) {
    container.innerHTML = '';
    const text = String(notesText || '').trim();
    if (!text) {
      const empty = document.createElement('p');
      empty.textContent = 'Tidak ada catatan perubahan untuk rilis ini.';
      container.append(empty);
      return;
    }
    const lines = text.split('\n');
    let currentUl = null;
    for (const rawLine of lines) {
      const line = rawLine.trim();
      if (!line) {
        currentUl = null;
        continue;
      }
      if (/^#\s+notely/i.test(line)) continue;
      if (/^#{1,3}\s+/.test(line)) {
        currentUl = null;
        const heading = document.createElement('div');
        heading.className = 'update-notes-heading';
        heading.textContent = line.replace(/^#{1,3}\s+/, '').trim();
        container.append(heading);
        continue;
      }
      if (/^[-*]\s+/.test(line)) {
        if (!currentUl) {
          currentUl = document.createElement('ul');
          currentUl.className = 'update-notes-list';
          container.append(currentUl);
        }
        const itemText = line.replace(/^[-*]\s+/, '').trim();
        const li = document.createElement('li');
        const colon = itemText.indexOf(':');
        if (colon > 0 && colon < 35) {
          const bold = document.createElement('strong');
          bold.textContent = itemText.slice(0, colon + 1);
          const rest = document.createTextNode(' ' + itemText.slice(colon + 1).trimStart());
          li.append(bold, rest);
        } else {
          li.textContent = itemText;
        }
        currentUl.append(li);
        continue;
      }
      currentUl = null;
      const p = document.createElement('p');
      p.textContent = line;
      container.append(p);
    }
  }

  async function check() {
    if (checking || document.getElementById('notely-update-dialog')) return;
    checking = true;
    try {
      const update = await tauri.core.invoke('check_update');
      if (update) show(update);
    } catch (error) {
      console.warn('Pemeriksaan update belum tersedia:', error);
    } finally { checking = false; }
  }

  function show(update) {
    const dialog = document.createElement('dialog');
    dialog.id = 'notely-update-dialog';
    dialog.className = 'notely-update-dialog';

    const header = document.createElement('div');
    header.className = 'update-header';

    const icon = document.createElement('div');
    icon.className = 'update-header-icon';
    icon.innerHTML = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>';

    const titleGroup = document.createElement('div');
    titleGroup.className = 'update-header-text';
    const title = document.createElement('h2');
    title.textContent = `Notely ${update.version} tersedia`;

    const versions = document.createElement('div');
    versions.className = 'update-versions';
    const currTag = document.createElement('span');
    currTag.className = 'update-tag current';
    currTag.textContent = `Versi saat ini: v${update.currentVersion}`;
    const arrow = document.createElement('span');
    arrow.className = 'update-tag-arrow';
    arrow.textContent = '➔';
    const newTag = document.createElement('span');
    newTag.className = 'update-tag new';
    newTag.textContent = `Versi baru: v${update.version}`;
    versions.append(currTag, arrow, newTag);

    titleGroup.append(title, versions);
    header.append(icon, titleGroup);

    const notesCard = document.createElement('div');
    notesCard.className = 'update-notes-card';
    renderReleaseNotes(update.notes, notesCard);

    const desc = document.createElement('p');
    desc.className = 'update-desc';
    desc.textContent = 'Update mengunduh installer, menutup Notely, lalu membuka instalasi pada folder yang sama.';

    const progressArea = document.createElement('div');
    progressArea.className = 'update-progress-area';
    progressArea.hidden = true;

    const progressTrack = document.createElement('div');
    progressTrack.className = 'update-progress-track';
    const progressFill = document.createElement('div');
    progressFill.className = 'update-progress-fill';
    progressTrack.append(progressFill);

    const progressMetrics = document.createElement('div');
    progressMetrics.className = 'update-progress-metrics';
    const sizeSpan = document.createElement('span');
    sizeSpan.textContent = 'Menghubungkan…';
    const speedSpan = document.createElement('span');
    progressMetrics.append(sizeSpan, speedSpan);

    const statusMsg = document.createElement('p');
    statusMsg.className = 'update-status-msg';
    statusMsg.setAttribute('role', 'status');
    statusMsg.textContent = 'Menyiapkan unduhan…';

    progressArea.append(progressTrack, progressMetrics, statusMsg);

    const actions = document.createElement('div');
    actions.className = 'update-actions';
    const later = document.createElement('button');
    later.className = 'button update-btn-later';
    later.textContent = 'Nanti saja';
    const install = document.createElement('button');
    install.className = 'button accent update-btn-install';
    install.textContent = 'Update Sekarang';
    actions.append(later, install);

    later.onclick = () => { dialog.close(); dialog.remove(); };
    let installing = false;
    dialog.addEventListener('cancel', e => { if (installing) e.preventDefault(); });
    dialog.addEventListener('close', () => { if (!installing) dialog.remove(); });

    install.onclick = async () => {
      if (busy()) {
        statusMsg.textContent = 'Selesaikan rekaman, transkripsi, ringkasan, atau unduhan model terlebih dahulu.';
        progressArea.hidden = false;
        return;
      }
      installing = true;
      install.disabled = later.disabled = true;
      progressArea.hidden = false;
      progressFill.style.width = '0%';
      sizeSpan.textContent = 'Menghubungkan…';
      speedSpan.textContent = '';
      statusMsg.textContent = 'Mengunduh dan memverifikasi installer…';
      install.textContent = 'Mengunduh…';

      let unlisten;
      let lastTime = 0;
      let lastBytes = 0;
      let smoothedSpeed = 0;

      try {
        unlisten = await tauri.event.listen('notely-update-progress', ({payload}) => {
          const now = performance.now();
          const received = payload?.received || 0;
          const total = payload?.total || null;
          if (!lastTime) {
            lastTime = now;
            lastBytes = received;
          } else {
            const dt = (now - lastTime) / 1000;
            if (dt >= 0.25) {
              const instantSpeed = (received - lastBytes) / dt;
              smoothedSpeed = smoothedSpeed === 0 ? instantSpeed : (smoothedSpeed * 0.65 + instantSpeed * 0.35);
              lastTime = now;
              lastBytes = received;
            }
          }
          if (total && total > 0) {
            const pct = Math.min(100, Math.round((received / total) * 100));
            progressFill.style.width = pct + '%';
            sizeSpan.textContent = `${formatBytes(received)} / ${formatBytes(total)} (${pct}%)`;
            speedSpan.textContent = formatSpeed(smoothedSpeed);
            statusMsg.textContent = pct >= 100 ? 'Memverifikasi signature dan menyiapkan instalasi…' : 'Mengunduh installer…';
          } else {
            sizeSpan.textContent = `${formatBytes(received)} diunduh`;
            speedSpan.textContent = formatSpeed(smoothedSpeed);
            statusMsg.textContent = 'Mengunduh installer…';
          }
        });

        await tauri.core.invoke('install_update');
      } catch (error) {
        statusMsg.textContent = 'Update gagal: ' + error + '. Klik Update untuk mencoba lagi.';
        install.textContent = 'Coba Lagi';
        try {
          const latest = await tauri.core.invoke('check_update');
          if (!latest || latest.version !== update.version) {
            dialog.remove();
            if (latest) show(latest);
          }
        } catch (_) {}
      } finally {
        unlisten?.();
        installing = false;
        install.disabled = later.disabled = false;
      }
    };

    dialog.append(header, notesCard, desc, progressArea, actions);
    document.body.append(dialog);
    dialog.showModal();
    later.focus();
  }

  // Check GitHub releases at startup and retry periodically if the app was offline.
  // No persistent corner button is needed for automatic update discovery.
  setTimeout(() => check(), 3500);
  setInterval(() => check(), 6 * 60 * 60 * 1000);
})();
