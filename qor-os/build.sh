#! /usr/bin/bash
if test ! -d ../userland/bin
then
  mkdir ../userland/bin
fi

set programs slib libc term shell prog hello libc-test pwd basic cat ls clear mkdir checkers bmp ps

cd ../userland

for i in $programs
do
    cd $i
    make $argv
    cd ..
done

cd ../qor-os

sudo losetup /dev/loop11 hdd.dsk

sudo mount /dev/loop11 /mnt
sudo rm -rf /mnt/*
sudo cp -r ../userland/bin/ /mnt/bin/
sudo cp -r ../userland/root/ /mnt/

ls -aiS /mnt/bin
ls -aiS /mnt/root

sudo sync /mnt

sudo umount /mnt

sudo losetup -d /dev/loop11