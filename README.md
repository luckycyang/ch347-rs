# ch347-rs

rust lib for ch347f

features

- [x] GPIO
- [x] IIC
- [] SPI
- ~~- [] JTAG/SWD~~

## 为什么没有 uart

串口设备会自动被 `cdc_acm` 注册为 `ttyACMx` 设备你可以自个包裹 `serialport` 来实现 `emmbedded-hal` 或者使用别人包好的 `linux_embedded_hal`

## 重构说明

spi 模块基本测试通过后将先前的代码结构进行调整， 主要解决创建外设句柄需要 `&device` 的情况，主要目的是通过 `init` 保存 `USB Interface` 这个全局变量，底层都通过 `$crate::ch347::Interface` 提供的 `write/read` 实现。引入 `emmbedded-hal-interal` 管理外设，通过 `::new(Peri<P>, ...)` 方式创建外设。

希望如下优雅的访问

```
// init ch347
let p = hal::init();

let io1 = Output::new(p.IO1, speed, level);
let spi = Spi::new(p.SPI0, ..config);
```
