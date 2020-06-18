
![](/cycle.apng?raw=true)

![](/wiring.jpg?raw=true)

My rpi1 is running Arch Linux ARM. First get some tools, then enable i2c and
load up modules.

    pacman -S i2c-tools lm_sensors
    printf "dtparam=i2c_arm=on\n" >>/boot/config.txt
    printf "i2c-dev\ni2c-bcm2708\n" >/etc/modules-load.d/my-i2c.conf

Let group `users` r/w the i2c-device!

    printf 'SUBSYSTEM=="i2c-dev", GROUP="users", MODE="0660"\n' >/etc/udev/rules.d/my-i2c.rules

Cross-compiling is currently done in Docker, see the old-fashioned [Makefile](Makefile).

# Fixing WiFI Dropout Issues

I'm using an Edimax EW-7811Un wifi-dongle which frequently loses connection,
perhaps due to power saving, and failing to properly wake up? I tried using the
following `8192cu` module with the blacklisting of other drivers and of the
power management, and it seems to have solved the issue.

  https://github.com/pvaret/rtl8192cu-fixes
