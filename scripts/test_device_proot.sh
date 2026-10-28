#!/system/bin/sh
export PROOT_TMP_DIR=/data/user/0/com.hyunnism.deployinstant/files/tmp
LIB_DIR=$(pm path com.hyunnism.deployinstant | head -n 1 | sed 's/package://' | sed 's/base.apk/lib\/arm64/')
ROOTFS=/data/user/0/com.hyunnism.deployinstant/files/rootfs

$LIB_DIR/libproot.so -r $ROOTFS -b /dev -b /proc -b /sys -w / /bin/sh -c "export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin; node -e 'fetch(\"https://registry.npmjs.org\").then(r => console.log(\"HTTP \" + r.status)).catch(e => console.error(e))'"
