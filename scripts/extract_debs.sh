#!/system/bin/sh
export PROOT_TMP_DIR=/data/local/tmp
ROOTFS=/data/local/tmp/rootfs

/data/local/tmp/libpdn.so -0 -r $ROOTFS -b /dev -b /proc -b /sys -w / /bin/sh -c '
for f in /var/cache/apt/archives/*.deb; do
    echo "Extracting $f..."
    dpkg -x "$f" /
done
echo "Testing git and curl:"
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
git --version
curl --version | head -n 1
'

