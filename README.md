# ch347-rs

rust lib for ch347f

features

- [x] GPIO
- [x] IIC
- [] SPI
  ~~- [] JTAG/SWD~~

## 为什么没有 uart

串口设备会自动被 `cdc_acm` 注册为 `ttyACMx` 设备你可以自个包裹 `serialport` 来实现 `emmbedded-hal` 或者使用别人包好的 `linux_embedded_hal`
