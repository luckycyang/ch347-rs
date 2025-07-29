use ch347_rs::ch347::is_ch34x_device;
use nusb::transfer::{Direction, EndpointType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let device = nusb::list_devices()?
        .filter(is_ch34x_device)
        .next()
        .unwrap();

    let device_handle = device.open().unwrap();

    let config = device_handle.configurations().next().unwrap();

    let mut found = None;

    for interface in config.interfaces() {
        let interface_num = interface.interface_number();

        log::info!("interface num: {}", interface_num);

        let Some(desc) = interface.alt_settings().next() else {
            continue;
        };

        if !(desc.class() == 0xff && desc.subclass() == 0x00 && desc.protocol() == 0x00) {
            log::info!("skip {interface_num} with wrong class/subclass/protocol");
            continue;
        }

        let mut epin = None;
        let mut epout = None;

        for endpoint in desc.endpoints() {
            let address = endpoint.address();
            log::info!("Endpoint {address:#04x}");
            if endpoint.transfer_type() != EndpointType::Bulk {
                log::info!("skip endpoint {address:#04x}");
                continue;
            }

            if endpoint.direction() == Direction::In {
                epin = Some(address)
            } else {
                epout = Some(address)
            }
        }

        // have been found interface
        if let (Some(epin), Some(epout)) = (epin, epout) {
            found = Some((interface_num, epin, epout));
            break;
        }
    }

    let Some((interface_num, rpin, epout)) = found else {
        panic!("Not found ch347 interface, is that you in current mode");
    };

    log::info!(
        "Found ch347 current interface: \r\n\tinterface_num: {interface_num},\r\n\tepin: {rpin:#04x}\r\n\tepout: {epout:#04x}"
    );

    let _interface = device_handle.claim_interface(interface_num).unwrap();

    log::info!("config desc: {:?}", config);

    Ok(())
}
