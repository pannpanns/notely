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
      empty.style.color = '#7c8d83';
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
      if (/^(?:#{1,3}\s+|what'?s new:?|catatan pembaruan:?)/i.test(line)) {
        currentUl = null;
        const heading = document.createElement('div');
        heading.className = 'update-notes-heading';
        heading.style.cssText = 'font-weight:700;font-size:11.5px;color:#18786c;text-transform:uppercase;letter-spacing:0.6px;margin:10px 0 6px;';
        heading.textContent = line.replace(/^#{1,3}\s+/, '').replace(/:$/, '').trim();
        container.append(heading);
        continue;
      }
      if (/^[-*•]\s+/.test(line)) {
        if (!currentUl) {
          currentUl = document.createElement('ul');
          currentUl.className = 'update-notes-list';
          currentUl.style.cssText = 'margin:0 0 8px;padding-left:18px;list-style-type:disc;';
          container.append(currentUl);
        }
        const itemText = line.replace(/^[-*•]\s+/, '').trim();
        const li = document.createElement('li');
        li.style.cssText = 'margin:5px 0;line-height:1.55;';
        const colon = itemText.indexOf(':');
        if (colon > 0 && colon < 35) {
          const bold = document.createElement('strong');
          bold.style.cssText = 'color:#1a332a;font-weight:600;';
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
      p.style.cssText = 'margin:6px 0;line-height:1.55;';
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
    dialog.style.cssText = 'width:min(550px,92vw);border:1px solid #dce6e1;border-radius:20px;padding:26px 28px;color:#1e2d27;background:#ffffff;box-shadow:0 24px 64px rgba(16,40,35,0.18);box-sizing:border-box;margin:auto;';

    const header = document.createElement('div');
    header.className = 'update-header';
    header.style.cssText = 'display:flex;align-items:flex-start;gap:14px;margin-bottom:16px;';

    const icon = document.createElement('div');
    icon.className = 'update-header-icon';
    icon.style.cssText = 'width:44px;height:44px;border-radius:12px;background:#eaf5f0;color:#18786c;display:grid;place-items:center;flex-shrink:0;';
    icon.innerHTML = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="width:22px;height:22px;"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>';

    const titleGroup = document.createElement('div');
    titleGroup.className = 'update-header-text';
    const title = document.createElement('h2');
    title.style.cssText = 'font-size:18px;font-weight:700;color:#1a3830;letter-spacing:-0.3px;margin:0 0 6px;';
    title.textContent = `Notely ${update.version} tersedia`;

    const versions = document.createElement('div');
    versions.className = 'update-versions';
    versions.style.cssText = 'display:flex;align-items:center;gap:8px;font-size:12px;flex-wrap:wrap;';
    const currTag = document.createElement('span');
    currTag.className = 'update-tag current';
    currTag.style.cssText = 'padding:2px 8px;border-radius:6px;background:#f0f4f2;color:#556e63;font-weight:550;';
    currTag.textContent = `Versi saat ini: v${update.currentVersion}`;
    const arrow = document.createElement('span');
    arrow.className = 'update-tag-arrow';
    arrow.style.cssText = 'color:#8fa89d;font-size:11px;';
    arrow.textContent = '➔';
    const newTag = document.createElement('span');
    newTag.className = 'update-tag new';
    newTag.style.cssText = 'padding:2px 8px;border-radius:6px;background:#e3f5ec;color:#18786c;font-weight:600;';
    newTag.textContent = `Versi baru: v${update.version}`;
    versions.append(currTag, arrow, newTag);

    titleGroup.append(title, versions);
    header.append(icon, titleGroup);

    const notesCard = document.createElement('div');
    notesCard.className = 'update-notes-card';
    notesCard.style.cssText = 'background:#f8fbf9;border:1px solid #e1ebe5;border-radius:12px;padding:14px 18px;margin:0 0 16px;max-height:190px;overflow-y:auto;font-size:12.5px;line-height:1.6;color:#334e44;box-sizing:border-box;';
    renderReleaseNotes(update.notes, notesCard);

    const footer = document.createElement('div');
    footer.className = 'update-footer';
    footer.style.cssText = 'border-top:1px solid #e7eee9;padding-top:16px;display:flex;flex-direction:column;gap:14px;';

    const desc = document.createElement('p');
    desc.className = 'update-desc';
    desc.style.cssText = 'font-size:12px;color:#677e73;line-height:1.55;margin:0;display:flex;align-items:flex-start;gap:7px;';
    desc.innerHTML = '<svg style="width:15px;height:15px;margin-top:1px;color:#18786c;flex-shrink:0;" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg><span>Update akan mengunduh installer, menutup Notely, lalu memasang pembaruan pada folder yang sama.</span>';

    const progressArea = document.createElement('div');
    progressArea.className = 'update-progress-area';
    progressArea.hidden = true;
    progressArea.style.cssText = 'background:#f2f7f4;border:1px solid #dfe9e3;border-radius:10px;padding:12px 14px;';

    const progressTrack = document.createElement('div');
    progressTrack.className = 'update-progress-track';
    progressTrack.style.cssText = 'height:8px;background:#d8e5dd;border-radius:999px;overflow:hidden;';
    const progressFill = document.createElement('div');
    progressFill.className = 'update-progress-fill';
    progressFill.style.cssText = 'height:100%;width:0%;background:linear-gradient(90deg,#18786c,#26a595);border-radius:999px;transition:width 0.25s ease-out;';
    progressTrack.append(progressFill);

    const progressMetrics = document.createElement('div');
    progressMetrics.className = 'update-progress-metrics';
    progressMetrics.style.cssText = 'display:flex;justify-content:space-between;align-items:center;font-size:12px;font-weight:600;color:#244439;margin-top:8px;font-variant-numeric:tabular-nums;';
    const sizeSpan = document.createElement('span');
    sizeSpan.textContent = 'Menghubungkan…';
    const speedSpan = document.createElement('span');
    progressMetrics.append(sizeSpan, speedSpan);

    const statusMsg = document.createElement('p');
    statusMsg.className = 'update-status-msg';
    statusMsg.setAttribute('role', 'status');
    statusMsg.style.cssText = 'font-size:11.5px;color:#5c756b;margin:6px 0 0;';
    statusMsg.textContent = 'Menyiapkan unduhan…';

    progressArea.append(progressTrack, progressMetrics, statusMsg);

    const actions = document.createElement('div');
    actions.className = 'update-actions';
    actions.style.cssText = 'display:flex;justify-content:flex-end;align-items:center;gap:10px;width:100%;margin-top:2px;';
    const later = document.createElement('button');
    later.className = 'button update-btn-later';
    later.style.cssText = 'height:38px;padding:0 18px;border-radius:9px;font-size:13px;font-weight:600;background:#f1f5f3;border:1px solid #d4dfd8;color:#445e53;cursor:pointer;transition:background 0.15s;';
    later.textContent = 'Nanti saja';
    const install = document.createElement('button');
    install.className = 'button accent update-btn-install';
    install.style.cssText = 'height:38px;padding:0 22px;border-radius:9px;font-size:13px;font-weight:600;background:#18786c;border:none;color:#ffffff;cursor:pointer;box-shadow:0 2px 8px rgba(24,120,108,0.28);transition:all 0.15s;';
    install.textContent = 'Update Sekarang';
    actions.append(later, install);

    footer.append(desc, progressArea, actions);

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

    dialog.append(header, notesCard, footer);
    document.body.append(dialog);
    dialog.showModal();
    later.focus();
  }

  // Check GitHub releases at startup and retry periodically if the app was offline.
  // No persistent corner button is needed for automatic update discovery.
  setTimeout(() => check(), 3500);
  setInterval(() => check(), 6 * 60 * 60 * 1000);
})();
