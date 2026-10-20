/* Native updater; release notes are always treated as text, never HTML. */
(() => {
  const tauri = window.__TAURI__;
  if (!tauri?.core?.invoke) return;
  let checking = false;
  const busy = () => typeof state !== 'undefined' && (state.recording || state.summaryLoading) ||
    typeof engine !== 'undefined' && (engine.state === 'stopping' || engine.transcribeJob || engine.currentJob || engine.modelJob || engine.ollamaJob || engine.cudaJob);
  async function check(manual = false) {
    if (checking || document.getElementById('notely-update-dialog')) return;
    checking = true;
    try {
      const update = await tauri.core.invoke('check_update');
      if (update) show(update);
      else if (manual) alert('Notely sudah menggunakan versi terbaru.');
    } catch (error) {
      if (manual) alert('Belum dapat memeriksa update. Periksa koneksi atau konfigurasi rilis.\n' + error);
      console.warn('Pemeriksaan update belum tersedia:', error);
    } finally { checking = false; }
  }
  function show(update) {
    const dialog = document.createElement('dialog');
    dialog.id = 'notely-update-dialog';
    dialog.style.cssText = 'width:min(560px,90vw);border:1px solid #dce5e5;border-radius:16px;padding:26px;color:#203039';
    const title = document.createElement('h2'); title.textContent = `Notely ${update.version} tersedia`;
    const version = document.createElement('p'); version.textContent = `Versi saat ini: ${update.currentVersion}`;
    const notes = document.createElement('pre'); notes.textContent = update.notes || 'Tidak ada catatan perubahan untuk rilis ini.';
    notes.style.cssText = 'white-space:pre-wrap;max-height:40vh;overflow:auto;font:inherit;line-height:1.6';
    const status = document.createElement('p'); status.setAttribute('role', 'status');
    status.textContent = 'Update mengunduh installer, menutup Notely, lalu membuka instalasi pada folder yang sama.';
    const later = document.createElement('button'); later.className = 'button'; later.textContent = 'Nanti saja';
    const install = document.createElement('button'); install.className = 'button accent'; install.textContent = 'Update';
    later.onclick = () => { dialog.close(); dialog.remove(); };
    let installing = false;
    dialog.addEventListener('cancel', e => { if (installing) e.preventDefault(); });
    dialog.addEventListener('close', () => { if (!installing) dialog.remove(); });
    install.onclick = async () => {
      if (busy()) { status.textContent = 'Selesaikan rekaman, transkripsi, ringkasan, atau unduhan model terlebih dahulu.'; return; }
      installing = true; install.disabled = later.disabled = true;
      let unlisten;
      try {
        unlisten = await tauri.event.listen('notely-update-progress', ({payload}) => {
          status.textContent = payload.total ? `Mengunduh update: ${Math.min(100, Math.round(payload.received / payload.total * 100))}%` : `Mengunduh: ${(payload.received / 1048576).toFixed(1)} MB`;
        });
        status.textContent = 'Mengunduh dan memverifikasi installer…';
        await tauri.core.invoke('install_update');
      } catch (error) {
        status.textContent = 'Update gagal: ' + error + '. Klik Update untuk mencoba lagi.';
        try { const latest = await tauri.core.invoke('check_update'); if (!latest || latest.version !== update.version) { dialog.remove(); if (latest) show(latest); } } catch (_) {}
      } finally { unlisten?.(); installing = false; install.disabled = later.disabled = false; }
    };
    dialog.append(title, version, notes, status, later, install);
    document.body.append(dialog); dialog.showModal(); later.focus();
  }
  // A small persistent entry point also supports retry after offline startup.
  const button = document.createElement('button'); button.textContent = 'Cek update';
  button.className = 'button'; button.title = 'Periksa pembaruan Notely';
  button.style.cssText = 'position:fixed;right:16px;bottom:8px;z-index:40;font-size:11px;min-height:26px';
  button.onclick = () => check(true); document.body.append(button);
  setTimeout(() => check(), 3500);
})();
