# rustfat32
This is a minimalistic fat32 reimplementation in rust for a school project youhouuu

To make this project work, create a "image" directory at the root of the project :

``` mkdir image ```

Then, navigate in the directory with `cd image` and execute these commands to format a shallow fat32.img file into a Fat32 device: 

```
dd if=/dev/zero of=fat32.img bs=1M count=16
mkfs.fat -F 32 fat32.img
```

then, mount the file with these commands : 

```
sudo mount -o loop fat32.img /mnt
ls /mnt
sudo mkdir /mnt/TEST
sudo touch /mnt/HELLO.TXT
ls /mnt
sudo umount /mnt
```
