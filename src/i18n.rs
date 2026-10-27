#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Id,
}

impl Default for Lang {
    fn default() -> Self {
        Lang::En
    }
}

impl Lang {
    pub fn key(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Id => "id",
        }
    }

    pub fn from_key(key: &str) -> Self {
        match key {
            "id" => Lang::Id,
            _ => Lang::En,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Id => "Bahasa Indonesia",
        }
    }
}

/// Tiny translation table keyed by the English source string. Unknown strings
/// intentionally fall back to English so new UI text never breaks.
pub fn t(lang: Lang, en: &'static str) -> &'static str {
    if lang == Lang::En {
        return en;
    }
    match en {
        // Ribbon tabs
        "Home" => "Home",
        "Scan" => "Pindai",
        "View" => "Tampilan",
        "Options" => "Opsi",
        // Groups
        "Operations" => "Operasi",
        "Mode" => "Mode",
        "Unit" => "Satuan",
        "Decimals" => "Desimal",
        "Folder Operations" => "Operasi Folder",
        "Filter" => "Filter",
        "Directory Tree" => "Pohon Direktori",
        "Tools" => "Alat",
        "Visible columns" => "Kolom terlihat",
        "Sort" => "Urutkan",
        "Show or hide" => "Tampilkan",
        "Language" => "Bahasa",
        "Application" => "Aplikasi",
        // Buttons
        "Select Directory" => "Pilih Folder",
        "Stop Scan" => "Hentikan Pindai",
        "Refresh" => "Segarkan",
        "Size" => "Ukuran",
        "Allocated Space" => "Ruang Terpakai",
        "File Count" => "Jumlah File",
        "Percent" => "Persentase",
        "Automatic Units" => "Satuan Otomatis",
        "Expand" => "Bentangkan",
        "New Folder" => "Folder Baru",
        "Clear Filter" => "Hapus Filter",
        "Hide empty folders" => "Sembunyikan folder kosong",
        "Search Tree" => "Cari di Pohon",
        "Parent Folder" => "Folder Induk",
        "Export" => "Ekspor",
        "Open Location" => "Buka Lokasi",
        "Configure columns" => "Atur Kolom",
        "Show Details" => "Tampilkan Detail",
        "Show Treemap Chart" => "Tampilkan Grafik Treemap",
        "Start as Admin" => "Sebagai Admin",
        "Full expand" => "Bentangkan Semua",
        "Collapse all" => "Ciutkan Semua",
        "Level 1" => "Level 1",
        "Level 2" => "Level 2",
        "Level 3" => "Level 3",
        "Level 4" => "Level 4",
        "Level 5" => "Level 5",
        "Details" => "Detail",
        "Treemap Chart" => "Grafik Treemap",
        // Columns
        "Name" => "Nama",
        "Allocated" => "Terpakai",
        "Files" => "File",
        "Folders" => "Folder",
        "% of Parent" => "% dari Induk",
        "Last Modified" => "Terakhir Diubah",
        // Status bar
        "Free Space" => "Ruang Kosong",
        "of" => "dari",
        "Excluded" => "Dikecualikan",
        "Bytes per Cluster" => "Byte per Kluster",
        "items" => "item",
        "files" => "file",
        "folders" => "folder",
        // Welcome / progress
        "Select a folder to start scanning" => "Pilih folder untuk mulai memindai",
        "Analyze disk usage of a folder" => "Analisis pemakaian disk pada sebuah folder",
        "No scan yet" => "Belum ada pemindaian",
        "Scanning..." => "Memindai...",
        "Stopping..." => "Menghentikan...",
        "Scan complete" => "Pemindaian selesai",
        "Scan stopped - partial results" => "Pemindaian dihentikan - hasil sebagian",
        "Scan failed" => "Pemindaian gagal",
        "Preparing scan" => "Menyiapkan pemindaian",
        "Ready" => "Siap",
        "Scanning" => "Memindai",
        // Dialogs
        "Filter files" => "Filter file",
        "Include (masks, separated by ;)" => "Sertakan (mask, dipisah ;)",
        "Exclude (masks, separated by ;)" => "Kecualikan (mask, dipisah ;)",
        "Minimum size (MB)" => "Ukuran minimum (MB)",
        "Maximum age (days, 0 = off)" => "Umur maksimum (hari, 0 = nonaktif)",
        "Apply" => "Terapkan",
        "Cancel" => "Batal",
        "Reset" => "Atur Ulang",
        "Close" => "Tutup",
        "Reset to defaults" => "Kembalikan ke bawaan",
        "New folder" => "Folder baru",
        "Folder name" => "Nama folder",
        "Create" => "Buat",
        "Parent:" => "Induk:",
        "Search in tree" => "Cari di pohon",
        "Search" => "Cari",
        "Next result" => "Hasil berikutnya",
        "No matches" => "Tidak ada hasil",
        "Copy path" => "Salin path",
        "Filter active" => "Filter aktif",
        "Scanned" => "Dipindai",
        "Up" => "Naik",
        "Root" => "Akar",
        "File" => "File",
        "Level" => "Level",
        "Double-click to zoom" => "Klik dua kali untuk zoom",
        "Rescan" => "Pindai ulang",
        "Open in Explorer" => "Buka di Explorer",
        "What is this folder?" => "Folder Apa ini?",
        "What is this file?" => "File Apa ini?",
        "Opening Google search" => "Membuka pencarian Google",
        "Could not open browser" => "Tidak dapat membuka browser",
        "Language changes take effect immediately" => "Perubahan bahasa langsung berlaku",
        // Misc
        "Size on disk" => "Ukuran di disk",
        "Total" => "Total",
        "Selected" => "Dipilih",
        "of total" => "dari total",
        "All" => "Semua",
        "Matches" => "Hasil",
        "Select a folder" => "Pilih folder",
        "Type a name to search" => "Ketik nama untuk mencari",
        "Show the Size column" => "Tampilkan kolom Ukuran",
        "Show the Allocated Space column" => "Tampilkan kolom Ruang Terpakai",
        "Show the File Count column" => "Tampilkan kolom Jumlah File",
        "Show the Percent column" => "Tampilkan kolom Persentase",
        "Automatically choose the best unit" => "Pilih satuan terbaik secara otomatis",
        "Expand the tree" => "Bentangkan pohon",
        "Create a new folder here" => "Buat folder baru di sini",
        "Filter files and folders" => "Filter file dan folder",
        "Remove all filters" => "Hapus semua filter",
        "Hide folders without visible files" => "Sembunyikan folder tanpa file terlihat",
        "Search for a folder in the tree" => "Cari folder di pohon",
        "Go to parent folder" => "Pergi ke folder induk",
        "Export visible rows as CSV" => "Ekspor baris terlihat sebagai CSV",
        "Open the selected item in Explorer" => "Buka item terpilih di Explorer",
        "Sort the tree" => "Urutkan pohon",
        "Show the details tree" => "Tampilkan pohon detail",
        "Show the treemap chart" => "Tampilkan grafik treemap",
        "Change the application language" => "Ubah bahasa aplikasi",
        "Restart with administrator rights" => "Mulai ulang dengan hak administrator",
        "Could not create folder" => "Tidak dapat membuat folder",
        "Could not export the file" => "Tidak dapat mengekspor file",
        // Permanent delete
        "Delete permanently" => "Hapus permanen",
        "Delete permanently?" => "Hapus permanen?",
        "Delete" => "Hapus",
        "Deleted" => "Dihapus",
        "Could not delete" => "Tidak dapat menghapus",
        "The scan root cannot be deleted" => {
            "Folder yang sedang dipindai tidak dapat dihapus"
        }
        "This item will be deleted permanently. This action cannot be undone." => {
            "Item ini akan dihapus permanen. Tindakan ini tidak dapat dibatalkan."
        }
        "Deleting a folder also deletes all of its contents." => {
            "Menghapus folder juga menghapus seluruh isinya."
        }
        "The mask field uses * and ? wildcards" => "Kolom mask memakai wildcard * dan ?",
        "decimals" => "desimal",
        "Stop" => "Hentikan",
        "Use Details or Treemap Chart from the View tab" => {
            "Gunakan Detail atau Grafik Treemap dari tab Tampilan"
        }
        _ => en,
    }
}
