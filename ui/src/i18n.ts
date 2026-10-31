// i18n.ts
// Ringan & tanpa dependensi eksternal, sesuai PRD Section 11 & filosofi /ponytail

export type Language = 'id' | 'en';

export const translations = {
  id: {
    appName: 'Deploy Instant',
    appSubtitle: 'Self-hosted PaaS Android',
    statusReady: 'Siap',
    statusInstalling: 'Memasang...',
    statusNotReady: 'Belum Terpasang',
    toolchainHeader: 'Node.js LTS • PM2 • Debian Rootfs',
    toolchainSummary: 'Toolchain Siap',
    projectsTab: 'Project',
    settingsTab: 'Pengaturan',
    diagnosticsTab: 'Diagnostik',
    
    // Dashboard
    activeEngine: 'Instant Node Engine',
    uptime: 'Status',
    servicesCount: 'Project Aktif',
    storageUsed: 'Ruang Rootfs',
    newProjectBtn: '+ Project Baru',
    projectListHeader: 'Daftar Project',
    noProjectsTitle: 'Belum Ada Project',
    noProjectsDesc: 'Tambahkan repository Git pertama Anda untuk memulai hosting aplikasi mandiri di perangkat ini.',
    createFirstProject: '+ Tambah Project Baru',
    
    // Project Card
    cardBranch: 'Branch',
    cardPort: 'Port',
    cardBuild: 'Build',
    cardStart: 'Start',
    autoStartBadge: 'Auto-Start',
    editBtn: 'Ubah',
    deleteBtn: 'Hapus',
    
    // Form Modal
    modalCreateTitle: 'Tambah Project Baru',
    modalEditTitle: 'Ubah Konfigurasi Project',
    modalClose: 'Tutup',
    modalCancel: 'Batal',
    modalSave: 'Simpan Project',
    modalSaving: 'Menyimpan...',
    fieldBasicInfo: 'Informasi Repository',
    fieldName: 'Nama Project',
    fieldNamePlaceholder: 'contoh: my-express-api',
    fieldRepoUrl: 'Git Repository URL',
    fieldRepoPlaceholder: 'https://github.com/user/repo.git',
    fieldBranch: 'Branch Git',
    fieldBranchPlaceholder: 'main',
    fieldExecution: 'Konfigurasi Eksekusi',
    fieldPort: 'Port Internal (Opsional)',
    fieldPortPlaceholder: 'contoh: 3000',
    fieldBuildCmd: 'Perintah Build',
    fieldBuildCmdPlaceholder: 'npm install',
    fieldStartCmd: 'Perintah Start',
    fieldStartCmdPlaceholder: 'npm start',
    fieldAutoStart: 'Auto-Start saat sistem siap',
    fieldEnvVars: 'Environment Variables',
    addEnvBtn: '+ Tambah Variabel',
    envKeyPlaceholder: 'KEY (misal: PORT)',
    envValPlaceholder: 'VALUE',
    
    // Engine Detail Modal
    engineModalTitle: 'Detail Engine & Toolchain',
    engineInfoSubtitle: 'Debian Bookworm arm64 Runtime',
    monitoringSection: 'Dependensi Terpasang',
    installEngineBtn: 'Pasang Lingkungan Server',
    reverifyBtn: 'Periksa Ulang Toolchain',
    engineNotInstalled: 'Lingkungan server belum terpasang di perangkat ini.',
    engineInstalledDesc: 'Seluruh dependensi utama siap menjalankan aplikasi Node.js mandiri.',
    viewDetailPill: 'Detail ›',
    installActionPill: 'Pasang ›',
    
    // Delete Confirmation Modal
    modalDeleteTitle: 'Hapus Project?',
    modalDeleteMsg: 'Apakah Anda yakin ingin menghapus project ini? Seluruh konfigurasi dan data project akan dihapus secara permanen.',
    confirmDeleteBtn: 'Ya, Hapus Project',
    
    // Settings
    settingsTitle: 'Pengaturan Aplikasi',
    languageSection: 'Bahasa Tampilan',
    langIndonesian: 'Bahasa Indonesia (ID)',
    langEnglish: 'English (EN)',
    aboutSection: 'Tentang Aplikasi',
    appVersion: 'Versi',
    engineRuntime: 'Engine Runtime',
    zeroLaptopNotice: 'Deploy Instant berjalan 100% mandiri di perangkat lokal Anda tanpa memerlukan laptop.',
    openDiagnostics: 'Buka Diagnostik Sistem',
    
    // Diagnostics (Fase 1 & Fase 2)
    diagnosticsTitle: 'Diagnostik Toolchain & Proot',
    probeBtn: 'Jalankan Probe Proot',
    verifyBtn: 'Verifikasi Toolchain',
    backToDashboard: '← Kembali ke Dashboard',
  },
  en: {
    appName: 'Deploy Instant',
    appSubtitle: 'Self-hosted Android PaaS',
    statusReady: 'Ready',
    statusInstalling: 'Installing...',
    statusNotReady: 'Not Ready',
    toolchainHeader: 'Node.js LTS • PM2 • Debian Rootfs',
    toolchainSummary: 'Toolchain Ready',
    projectsTab: 'Projects',
    settingsTab: 'Settings',
    diagnosticsTab: 'Diagnostics',
    
    // Dashboard
    activeEngine: 'Instant Node Engine',
    uptime: 'Status',
    servicesCount: 'Active Projects',
    storageUsed: 'Rootfs Space',
    newProjectBtn: '+ New Project',
    projectListHeader: 'Projects List',
    noProjectsTitle: 'No Projects Yet',
    noProjectsDesc: 'Add your first Git repository to start self-hosting applications directly on this device.',
    createFirstProject: '+ Add New Project',
    
    // Project Card
    cardBranch: 'Branch',
    cardPort: 'Port',
    cardBuild: 'Build',
    cardStart: 'Start',
    autoStartBadge: 'Auto-Start',
    editBtn: 'Edit',
    deleteBtn: 'Delete',
    
    // Form Modal
    modalCreateTitle: 'New Project',
    modalEditTitle: 'Edit Project Configuration',
    modalClose: 'Close',
    modalCancel: 'Cancel',
    modalSave: 'Save Project',
    modalSaving: 'Saving...',
    fieldBasicInfo: 'Repository Information',
    fieldName: 'Project Name',
    fieldNamePlaceholder: 'e.g. my-express-api',
    fieldRepoUrl: 'Git Repository URL',
    fieldRepoPlaceholder: 'https://github.com/user/repo.git',
    fieldBranch: 'Git Branch',
    fieldBranchPlaceholder: 'main',
    fieldExecution: 'Execution Configuration',
    fieldPort: 'Internal Port (Optional)',
    fieldPortPlaceholder: 'e.g. 3000',
    fieldBuildCmd: 'Build Command',
    fieldBuildCmdPlaceholder: 'npm install',
    fieldStartCmd: 'Start Command',
    fieldStartCmdPlaceholder: 'npm start',
    fieldAutoStart: 'Auto-Start when engine ready',
    fieldEnvVars: 'Environment Variables',
    addEnvBtn: '+ Add Variable',
    envKeyPlaceholder: 'KEY (e.g. PORT)',
    envValPlaceholder: 'VALUE',
    
    // Engine Detail Modal
    engineModalTitle: 'Engine & Toolchain Details',
    engineInfoSubtitle: 'Debian Bookworm arm64 Runtime',
    monitoringSection: 'Installed Dependencies',
    installEngineBtn: 'Install Server Environment',
    reverifyBtn: 'Re-check Toolchain',
    engineNotInstalled: 'Server environment is not installed on this device.',
    engineInstalledDesc: 'All primary dependencies are ready to execute Node.js applications autonomously.',
    viewDetailPill: 'Details ›',
    installActionPill: 'Install ›',
    
    // Delete Confirmation Modal
    modalDeleteTitle: 'Delete Project?',
    modalDeleteMsg: 'Are you sure you want to delete this project? All configuration and project data will be permanently removed.',
    confirmDeleteBtn: 'Yes, Delete Project',
    
    // Settings
    settingsTitle: 'Application Settings',
    languageSection: 'Display Language',
    langIndonesian: 'Bahasa Indonesia (ID)',
    langEnglish: 'English (EN)',
    aboutSection: 'About Application',
    appVersion: 'Version',
    engineRuntime: 'Engine Runtime',
    zeroLaptopNotice: 'Deploy Instant runs 100% autonomously on your local device without requiring a laptop.',
    openDiagnostics: 'Open System Diagnostics',
    
    // Diagnostics (Fase 1 & Fase 2)
    diagnosticsTitle: 'Toolchain & Proot Diagnostics',
    probeBtn: 'Run Proot Probe',
    verifyBtn: 'Verify Toolchain',
    backToDashboard: '← Back to Dashboard',
  },
} as const;

export type TranslationKey = keyof typeof translations.id;

