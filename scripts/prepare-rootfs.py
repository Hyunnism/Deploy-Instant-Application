#!/usr/bin/env python3
"""
scripts/prepare-rootfs.py
Menggabungkan Debian Bookworm arm64 + Node.js LTS ke dalam rootfs.tar.gz
yang dapat diekstrak langsung oleh /system/bin/tar di Android.
"""
import tarfile
import io
import os
import sys

assets_dir = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "assets")
debian_xz = os.path.join(assets_dir, "rootfs-min-arm64.tar.xz")
node_xz = os.path.join(assets_dir, "node-v20-linux-arm64.tar.xz")
out_gz = os.path.join(assets_dir, "rootfs.tar.gz")

if not os.path.exists(debian_xz) or not os.path.exists(node_xz):
    print("Error: Aset rootfs atau nodejs belum diunduh. Jalankan fetch-assets.ps1 dulu.")
    sys.exit(1)

print("==> Membuka Debian rootfs dan Node.js archive...")
with tarfile.open(debian_xz, "r:xz") as deb_tar, \
     tarfile.open(node_xz, "r:xz") as node_tar, \
     tarfile.open(out_gz, "w:gz") as out_tar:

    print("==> Menyalin komponen Debian rootfs (menghindari device nodes)...")
    prefix = "debian-bookworm-aarch64/"
    count = 0
    for member in deb_tar.getmembers():
        # Lewati device nodes di /dev (akan di-mount oleh proot -b /dev)
        if member.name.startswith(prefix + "dev/") and member.name != prefix + "dev/":
            continue
        
        # Strip prefix
        if member.name.startswith(prefix):
            member.name = member.name[len(prefix):]
        if not member.name:
            continue

        fileobj = deb_tar.extractfile(member) if member.isreg() else None
        out_tar.addfile(member, fileobj)
        count += 1
        if count % 2000 == 0:
            print(f"    Diproses: {count} file...")

    print(f"==> Total file Debian: {count}")

    print("==> Mengintegrasikan Node.js ke /usr/local...")
    node_prefix = "node-v20.18.0-linux-arm64/"
    node_count = 0
    for member in node_tar.getmembers():
        if not member.name.startswith(node_prefix):
            continue
        rel = member.name[len(node_prefix):]
        if not rel:
            continue
        member.name = "usr/local/" + rel
        fileobj = node_tar.extractfile(member) if member.isreg() else None
        out_tar.addfile(member, fileobj)
        node_count += 1

    print(f"==> Total file Node.js: {node_count}")

    # Tambahkan /etc/resolv.conf default
    resolv_content = b"nameserver 1.1.1.1\nnameserver 8.8.8.8\n"
    resolv_info = tarfile.TarInfo(name="etc/resolv.conf")
    resolv_info.size = len(resolv_content)
    resolv_info.mode = 0o644
    out_tar.addfile(resolv_info, io.BytesIO(resolv_content))
    print("==> /etc/resolv.conf disematkan")

print(f"==> [SUCCESS] rootfs.tar.gz berhasil dibuat di: {out_gz}")
print(f"    Ukuran: {os.path.getsize(out_gz) / (1024*1024):.2f} MB")

