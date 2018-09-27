
On the Arch Linux ARM isntall on my rpi1, I get some tools, enable i2c and load
up modules.

    pacman -S i2c-tools lm_sensors
    printf "dtparam=i2c_arm=on\n" >>/boot/config.txt
    printf "i2c-dev\ni2c-bcm2708\n" >/etc/modules-load.d/my-i2c.conf

Let group `users` r/w the i2c-device!

    printf 'SUBSYSTEM=="i2c-dev", GROUP="users", MODE="0660"\n' >/etc/udev/rules.d/my-i2c.rules

Cross-compiling is currently done in Docker, see the old-fashioned [Makefile](Makefile).
