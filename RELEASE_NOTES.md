Pembaruan Notely v1.2.7:
• Perbaikan Kisi Kalender Bawah Terpotong (Fix Calendar Row Truncation): Memperbaiki struktur kisi kalender dengan memisahkan baris nama hari dan kisi tanggal ke elemen mandiri serta meniadakan margin/padding bawah berlebih pada kontainer halaman. Bulan dengan 6 minggu (seperti November 2026) kini tampil utuh 100% dari tanggal 1 hingga akhir tanpa ada angka atau sel yang terpotong di bagian bawah.
• Perbaikan Tautan Google Cloud Console (100% Clickable): Memperbaiki link dan menambahkan tombol khusus `Buka Google Cloud Console: Credentials ↗` yang langsung membuka peramban resmi (Chrome/default) dengan aman dan andal melalui penanganan event native.
• Redesain Modal Kredensial Lapang & Bebas Sempit (Spacious Modal Card): Memperluas lebar modal dialog kredensial, memberikan ruang nafas (padding & margin lapang), serta menambahkan bilah gulir lembut internal (`max-height: 88vh; overflow-y: auto`) sehingga kartu tidak lagi mepet dengan tepi jendela laptop.
• Panduan Langkah Demi Langkah yang Ramah & Jelas: Menata ulang panduan Google Cloud ke dalam kartu langkah bernomor (Langkah 1 s.d. 5) yang secara gamblang menjelaskan pilihan 'External', penambahan email ke 'Test users' (mencegah Error 403), dan pemilihan tipe 'Desktop app'.
• Kartu Alternatif Mode Cepat di Bagian Teratas Dialog: Menyematkan kartu sorotan di bagian atas dialog yang mengingatkan pengguna awam bahwa mereka dapat langsung menggunakan Mode Cepat Web (1-klik) tanpa perlu repot berurusan dengan Google Cloud Console.

Pembaruan Notely v1.2.6:
• Perbaikan Peluncuran Browser Mode Cepat (Fix File Explorer Opening): Mengganti pemanggilan shell Windows dari explorer.exe ke PowerShell Start-Process dan Python os.startfile sehingga link pembuatan acara Google Calendar langsung membuka peramban web (Chrome/default) secara andal tanpa membuka jendela File Explorer.
• Penyederhanaan Integrasi Google Calendar (Hilangkan OAuth Publik, Khusus Kredensial Pribadi): Menghapus alur login OAuth publik yang sering terhambat Error 403 Google Cloud. Menetapkan Mode Cepat Web (1-klik tanpa setup) sebagai mode default dan menyediakan opsi Kredensial Khusus Developer bagi pengguna yang memiliki Client ID & Secret Google Cloud pribadi.
• Perapian Desain & Tipografi Panel Pengaturan: Merapikan tata letak kartu pengaturan integrasi kalender, menghilangkan teks bertumpuk, menyelaraskan ikon, dan memberikan tampilan visual yang bersih, modern, dan proporsional.
• Pembaruan Otomatis Mode Timpa (In-Place Overwrite Upgrade): Penginstal kini mendukung pembaruan senyap (/S) sehingga pembaruan versi dapat langsung menimpa file Notely yang lama tanpa perlu repot uninstall manual atau kehilangan data.

Pembaruan Notely v1.2.5:
• Penghapusan Dropdown Riwayat Input (No Browser Autocomplete): Menambahkan proteksi atribut autofill pada seluruh formulir penambahan tugas dan pencarian, mencegah munculnya pop-up dropdown hitam riwayat browser yang menutupi antarmuka.
• Peluncuran Andal Mode Cepat Google Calendar (Auto Open Chrome): Memperbaiki mekanisme peluncuran peramban web pada Windows menggunakan native handler sehingga Chrome langsung terbuka ke halaman pembuatan acara Google Calendar dengan data tugas terisi lengkap.
• Dialog Konfirmasi Hapus Tugas & Solusi 1-Klik: Menghilangkan bug klik 2x saat menghapus tugas dan menambahkan dialog konfirmasi yang aman dan rapi sebelum tugas dihapus permanen.
• Tampilan Kalender Terkunci (Fixed Viewport - No Scroll): Menyesuaikan kisi kalender bulanan agar pas 100% pada tinggi layar laptop tanpa memunculkan bilah gulir vertikal (scrollbar), memberikan pengalaman kalender desktop yang solid dan rapi.
• Kerapian Dialog Kredensial OAuth & Panduan Error 403: Menyembunyikan tombol default file picker bawaan peramban pada area drag-and-drop JSON dan menambahkan panduan penambahan Test Users di Google Cloud Console untuk mengatasi Google Error 403 access_denied.

Pembaruan Notely v1.2.4:
• Solusi Ganda Google Calendar (Mode Cepat Web & Mode Otomatis OAuth): Pengguna kini bebas memilih cara integrasi yang diinginkan:
  1. Mode Cepat via Web (⭐ Paling Mudah & Nol Setup): Tanpa izin akun atau kredensial developer, Notely langsung membuka Google Calendar di peramban dengan formulir judul dan tanggal tugas yang sudah otomatis terisi lengkap. Pengguna tinggal klik 'Simpan'.
  2. Mode Otomatis Latar Belakang (OAuth 2.0): Sinkronisasi penuh ke kalender HP & laptop tanpa membuka peramban, dilengkapi panduan visual interaktif yang super ramah dan mudah dipahami orang awam.
• Perbaikan Presisi Tata Letak Teks & Antarmuka (Anti-Sagging Text Fixes): Menyelaraskan seluruh optical vertical alignment pada tombol, badge, tab navigasi, dan indikator status. Menghilangkan ruang kosong ganda pada menu Pengaturan sehingga posisi judul dan deskripsi tidak lagi melorot terlalu ke bawah.
• Perbaikan Semantik Dialog Modal: Memperbaiki pembungkus elemen bodi dialog modal agar kompatibel dengan standar HTML tanpa penutupan tag otomatis yang memecah hierarki DOM.
