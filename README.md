# rustfat32
This is a minimalistic fat32 reimplementation in rust for a school project youhouuu

To make this project work, create a "image" directory at the root of the project :

``` mkdir image ```

Then, navigate in the directory with `cd image` and execute these commands to format a shallow fat32.img file into aFat32 device: 

```
dd if=/dev/zero of=fat32.img bs=1M count=10

mkfs.fat -F 32 fat32.img
```
