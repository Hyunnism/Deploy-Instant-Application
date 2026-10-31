#!/usr/bin/env python3
"""
scripts/prepare-rootfs.py
Menggabungkan:
1. Debian Bookworm arm64 (rootfs-min-arm64.tar.xz)
2. Paket resmi Git, Curl, CA-Certificates (debs/*.deb)
3. Node.js v24.22.1 LTS ARM64 (node-v24-linux-arm64.tar.xz)
4. PM2 Production Process Manager (pm2-staging/node_modules)
5. Cloudflared ARM64 (cloudflared-linux-arm64)
Menjadi satu arsip rootfs.tar.gz final Fase 2.
"""
import tarfile
import io
import os
import sys
import glob
import hashlib

assets_dir = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "assets")
debian_xz = os.path.join(assets_dir, "rootfs-min-arm64.tar.xz")
node_xz = os.path.join(assets_dir, "node-v24-linux-arm64.tar.xz")
cloudflared_bin = os.path.join(assets_dir, "cloudflared-linux-arm64")
pm2_dir = os.path.join(assets_dir, "pm2-staging", "node_modules")
debs_dir = os.path.join(assets_dir, "debs")
out_gz = os.path.join(assets_dir, "rootfs.tar.gz.tmp")
final_gz = os.path.join(assets_dir, "rootfs.tar.gz")

if not os.path.exists(debian_xz):
    print("Error: debian rootfs-min-arm64.tar.xz belum ada.")
    sys.exit(1)
if not os.path.exists(node_xz):
    print("Error: Node.js v24 belum diunduh.")
    sys.exit(1)
if not os.path.exists(cloudflared_bin):
    print("Error: Binary cloudflared belum diunduh.")
    sys.exit(1)

def extract_deb_data(deb_path):
    with open(deb_path, "rb") as f:
        magic = f.read(8)
        if magic != b"!<arch>\n":
            return None
        while True:
            header = f.read(60)
            if not header or len(header) < 60:
                break
            name = header[:16].strip().decode("ascii")
            size = int(header[48:58].strip())
            data = f.read(size)
            if size % 2 == 1:
                f.read(1)
            if name.startswith("data.tar"):
                mode = "r:xz" if name.endswith(".xz") else ("r:gz" if name.endswith(".gz") else "r:")
                return tarfile.open(fileobj=io.BytesIO(data), mode=mode)
    return None

print("==> Membuka Debian Bookworm dan Node.js v24 archive...")
with tarfile.open(debian_xz, "r:xz") as deb_tar, \
     tarfile.open(node_xz, "r:xz") as node_tar, \
     tarfile.open(out_gz, "w:gz", format=tarfile.USTAR_FORMAT) as out_tar:

    # 1. Salin Debian Bookworm base
    print("==> Menyalin komponen Debian Bookworm base...")
    prefix = "debian-bookworm-aarch64/"
    count = 0
    for member in deb_tar.getmembers():
        if member.name.startswith(prefix + "dev/") and member.name != prefix + "dev/":
            continue
        if member.name.startswith(prefix):
            member.name = member.name[len(prefix):]
        if not member.name or member.name.startswith("usr/local/") or member.name == "usr/local":
            continue
        fileobj = deb_tar.extractfile(member) if member.isreg() else None
        out_tar.addfile(member, fileobj)
        count += 1
        if count % 3000 == 0:
            print(f"    Diproses: {count} file...")
    print(f"==> Total file Debian base: {count}")

    # 2. Ekstrak deb packages (git, curl, ca-certificates, libssh2, etc.)
    deb_files = glob.glob(os.path.join(debs_dir, "*.deb"))
    print(f"==> Mengintegrasikan {len(deb_files)} paket deb (git, curl, dll)...")
    deb_count = 0
    for deb_path in deb_files:
        deb_tar_data = extract_deb_data(deb_path)
        if not deb_tar_data:
            continue
        for member in deb_tar_data.getmembers():
            # Hapus leading './' jika ada
            if member.name.startswith("./"):
                member.name = member.name[2:]
            if not member.name or member.name in (".", "lib", "bin", "sbin", "usr", "etc"):
                continue
            if member.name.startswith("usr/local/"):
                continue

            # Normalisasi Debian Merged-USR:
            # Cegah penimpaan symlink rootfs /lib, /bin, /sbin oleh direktori debian
            if member.name.startswith("lib/"):
                member.name = "usr/" + member.name
            elif member.name.startswith("bin/"):
                member.name = "usr/" + member.name
            elif member.name.startswith("sbin/"):
                member.name = "usr/" + member.name

            try:
                fileobj = deb_tar_data.extractfile(member) if member.isreg() else None
                out_tar.addfile(member, fileobj)
                deb_count += 1
            except Exception:
                pass
    print(f"==> Total file dari paket deb: {deb_count}")

    # 3. Integrasikan Node.js v24.22.1 LTS ke /usr/local
    print("==> Mengintegrasikan Node.js v24.22.1 LTS ke /usr/local...")
    node_prefix = "node-v24.22.1-linux-arm64/"
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
    print(f"==> Total file Node.js v24: {node_count}")

    # 4. Integrasikan PM2
    print("==> Mengintegrasikan PM2 ke /usr/local/lib/node_modules...")
    pm2_count = 0
    if os.path.exists(pm2_dir):
        for root, dirs, files in os.walk(pm2_dir):
            for f in files:
                full_path = os.path.join(root, f)
                rel_path = os.path.relpath(full_path, pm2_dir).replace("\\", "/")
                target_name = f"usr/local/lib/node_modules/{rel_path}"
                ti = tarfile.TarInfo(name=target_name)
                ti.size = os.path.getsize(full_path)
                ti.mode = 0o755 if f.endswith(".sh") or "bin" in rel_path else 0o644
                with open(full_path, "rb") as fp:
                    out_tar.addfile(ti, fp)
                pm2_count += 1
    print(f"==> Total file PM2: {pm2_count}")

    # 5. Symlink PM2
    print("==> Membuat symlink /usr/local/bin/pm2...")
    pm2_link = tarfile.TarInfo(name="usr/local/bin/pm2")
    pm2_link.type = tarfile.SYMTYPE
    pm2_link.linkname = "../lib/node_modules/pm2/bin/pm2"
    pm2_link.mode = 0o777
    out_tar.addfile(pm2_link)

    # 6. Integrasikan Cloudflared
    print("==> Mengintegrasikan Cloudflared arm64 ke /usr/local/bin/cloudflared...")
    cf_info = tarfile.TarInfo(name="usr/local/bin/cloudflared")
    cf_info.size = os.path.getsize(cloudflared_bin)
    cf_info.mode = 0o755
    with open(cloudflared_bin, "rb") as fp:
        out_tar.addfile(cf_info, fp)

    # 7. Resolv.conf & .version
    print("==> Menyematkan /etc/resolv.conf dan .version...")
    resolv_content = b"nameserver 1.1.1.1\nnameserver 8.8.8.8\n"
    resolv_info = tarfile.TarInfo(name="etc/resolv.conf")
    resolv_info.size = len(resolv_content)
    resolv_info.mode = 0o644
    out_tar.addfile(resolv_info, io.BytesIO(resolv_content))

    version_content = b"24.22.1-lts\n"
    version_info = tarfile.TarInfo(name=".version")
    version_info.size = len(version_content)
    version_info.mode = 0o644
    out_tar.addfile(version_info, io.BytesIO(version_content))

if os.path.exists(final_gz):
    os.remove(final_gz)
os.rename(out_gz, final_gz)

print(f"==> Menghitung checksum SHA-256...")
hasher = hashlib.sha256()
with open(final_gz, "rb") as f:
    while chunk := f.read(1024 * 1024):
        hasher.update(chunk)
sha256 = hasher.hexdigest()

print(f"==> [SUCCESS] Final rootfs.tar.gz Fase 2 selesai dibuat!")
print(f"    Ukuran: {os.path.getsize(final_gz) / (1024*1024):.2f} MB")
print(f"    SHA-256: {sha256}")

# Update rootfs/checksums.txt
checksums_path = os.path.join(os.path.dirname(__file__), "..", "rootfs", "checksums.txt")
with open(checksums_path, "w") as f:
    f.write(f"# Checksums untuk assets rootfs dan binary Fase 2 (Node.js v24.22.1 LTS + PM2 + Cloudflared + Git + Curl)\n")
    f.write(f"# Format: <SHA256>  <NAMA_FILE>\n\n")
    f.write(f"{sha256}  rootfs.tar.gz\n")
    f.write(f"47466f272c72bdfb6c68a4ee7f311dfc2825b03fbfa5ee0535e80cf91f13b63d  libproot.so\n")
    f.write(f"1934cfa729bc811f1a56bc342ae4e12c5bbf5ebfe83995f5dc4333dc5e1e1948  libproot-loader.so\n")

print(f"==> Checksum berhasil disimpan ke {checksums_path}")
