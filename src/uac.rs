use usb_device::{class_prelude::{UsbClass, UsbBus, InterfaceNumber, ControlOut, ControlIn, UsbBusAllocator}, endpoint::{Endpoint, In, Out, EndpointDirection, EndpointType}, control::{RequestType, Recipient, Request}, UsbError};

const AUDIO: u8 = 0x01;
const AUDIOCONTROL: u8 = 0x01;
const AUDIOSTREAMING: u8 = 0x02;
const CS_INTERFACE: u8 = 0x24;
const CS_ENDPOINT: u8 = 0x25;

pub struct AudioStream<'a, B, D>
where
    B: UsbBus,
    D: EndpointDirection,
{
    iface: InterfaceNumber,
    pub ep: Endpoint<'a, B, D>,
    alt_setting: u8,
}

pub struct UsbAudio<'a, B>
where
    B: UsbBus,
{
    control_iface: InterfaceNumber,
    pub input: AudioStream<'a, B, In>,
    pub output1: AudioStream<'a, B, Out>,
    pub output2: AudioStream<'a, B, Out>,
}

impl<'a, B> UsbAudio<'a, B>
where
    B: UsbBus,
{
    pub fn build(bus: &'a UsbBusAllocator<B>) -> Result<Self, UsbError> {
        let control_iface = bus.interface();
        let input = {
            let iface = bus.interface();
            let ep = bus.alloc(None, EndpointType::Isochronous, 192, 1)?;
            AudioStream {
                iface,
                ep,
                alt_setting: 0,
            }
        };
        let output1 = {
            let iface = bus.interface();
            let ep = bus.alloc(None, EndpointType::Isochronous, 192, 1)?;
            AudioStream {
                iface,
                ep,
                alt_setting: 0,
            }
        };
        let output2 = {
            let iface = bus.interface();
            let ep = bus.alloc(None, EndpointType::Isochronous, 192, 1)?;
            AudioStream {
                iface,
                ep,
                alt_setting: 0,
            }
        };
        Ok(Self {
            control_iface,
            input,
            output1,
            output2,
        })
    }
}

impl<B> UsbClass<B> for UsbAudio<'_, B>
where
    B: UsbBus,
{
    fn get_configuration_descriptors(&self, writer: &mut usb_device::descriptor::DescriptorWriter) -> usb_device::Result<()> {
        writer.interface(self.control_iface, AUDIO, AUDIOCONTROL, 0x00)?;

        let total_length: u16 = 8 + (1 + 9 + 12) * 3;
        writer.write(CS_INTERFACE, &[
            0x01, // HEADER,
            0x00,
            0x01, // bcd ADC
            total_length as u8,
            (total_length >> 8) as u8,
            0x03, // bInCollection
            self.input.iface.into(), // input if
            self.output1.iface.into(), // output1 if
            self.output2.iface.into(), // output2 if
        ])?;

        // input
        writer.write(CS_INTERFACE, &[
            0x02, // bDescriptorSubtype = INPUT_TERMINAL
            0x01, // bTerminalID
            0x01,
            0x02, // wTerminalType = USB Microphone
            0x00, // bAssocTerminal
            0x02, // bNrChannels
            0x03,
            0x00, // wChannelConfig = Left Front and Right Front
            0x00, // iChannelNames
            0x00, // iTerminal
        ])?;
        writer.write(CS_INTERFACE, &[
            0x03, // bDescriptorSubtype = OUTPUT_TERMINAL
            0x02, // bTerminalID
            0x01,
            0x01, // wTerminalType = USB Streaming
            0x00, // bAssocTerminal
            0x01, // bSourceID
            0x00, // iTerminal
        ])?;

        // output 1
        writer.write(CS_INTERFACE, &[
            0x02, // bDescriptorSubtype = INPUT_TERMINAL
            0x03, // bTerminalID
            0x01,
            0x01, // wTerminalType = USB Streaming
            0x00, // bAssocTerminal
            0x02, // bNrChannels
            0x03,
            0x00, // wChannelConfig = Left Front and Right Front
            0x00, // iChannelNames
            0x00, // iTerminal
        ])?;
        writer.write(CS_INTERFACE, &[
            0x03, // bDescriptorSubtype = OUTPUT_TERMINAL
            0x04, // bTerminalID
            0x01,
            0x03, // wTerminalType = Speaker
            0x00, // bAssocTerminal
            0x03, // bSourceID
            0x00, // iTerminal
        ])?;

        // output 2
        writer.write(CS_INTERFACE, &[
            0x02, // bDescriptorSubtype = INPUT_TERMINAL
            0x05, // bTerminalID
            0x01,
            0x01, // wTerminalType = USB Streaming
            0x00, // bAssocTerminal
            0x02, // bNrChannels
            0x03,
            0x00, // wChannelConfig = Left Front and Right Front
            0x00, // iChannelNames
            0x00, // iTerminal
        ])?;
        writer.write(CS_INTERFACE, &[
            0x03, // bDescriptorSubtype = OUTPUT_TERMINAL
            0x06, // bTerminalID
            0x01,
            0x03, // wTerminalType = Speaker
            0x00, // bAssocTerminal
            0x05, // bSourceID
            0x00, // iTerminal
        ])?;

        // input
        writer.interface(self.input.iface, AUDIO, AUDIOSTREAMING, 0x00)?;
        writer.interface_alt(self.input.iface, 0x01, AUDIO, AUDIOSTREAMING, 0x00, None)?;
        writer.write(
            CS_INTERFACE,
            &[
                0x01, // bDescriptorSubtype = AS_GENERAL
                0x02, // bTerminalLink = input
                0x01, // bDelay
                0x01,
                0x00, // wFormatTag = PCM
            ],
        )?;
        writer.write(
            CS_INTERFACE,
            &[
                0x02, // bDescriptorSubtype = FORMAT_TYPE
                0x01, // bFormatType = FORMAT_TYPE_I
                0x02, // bNrChannels = 2
                0x02, // bSubFrameSize = 2 (16bit)
                0x10, // bBitResolution = 16
                0x01, // bSamFreqType
                0x80, //
                0xbb, //
                0x00, // tSamFreq = 48k = 0xbb80
            ],
        )?;
        writer.endpoint(&self.input.ep)?;
        writer.write(
            CS_ENDPOINT,
            &[
                0x01, // bDescriptorSubtype = EP_GENERAL
                0x00, // bmAttributes
                0x00, // bLockDelayUnits
                0x00, //
                0x00, // wLockDelay
            ],
        )?;

        // output 1
        writer.interface(self.output1.iface, AUDIO, AUDIOSTREAMING, 0x00)?;
        writer.interface_alt(self.output1.iface, 0x01, AUDIO, AUDIOSTREAMING, 0x00, None)?;
        writer.write(
            CS_INTERFACE,
            &[
                0x01, // bDescriptorSubtype = AS_GENERAL
                0x03, // bTerminalLink = output 1
                0x01, // bDelay
                0x01,
                0x00, // wFormatTag = PCM
            ],
        )?;
        writer.write(
            CS_INTERFACE,
            &[
                0x02, // bDescriptorSubtype = FORMAT_TYPE
                0x01, // bFormatType = FORMAT_TYPE_I
                0x02, // bNrChannels = 2
                0x02, // bSubFrameSize = 2 (16bit)
                0x10, // bBitResolution = 16
                0x01, // bSamFreqType
                0x80, //
                0xbb, //
                0x00, // tSamFreq = 48k = 0xbb80
            ],
        )?;
        writer.endpoint(&self.output1.ep)?;
        writer.write(
            CS_ENDPOINT,
            &[
                0x01, // bDescriptorSubtype = EP_GENERAL
                0x00, // bmAttributes
                0x00, // bLockDelayUnits
                0x00, //
                0x00, // wLockDelay
            ],
        )?;

        // output 2
        writer.interface(self.output2.iface, AUDIO, AUDIOSTREAMING, 0x00)?;
        writer.interface_alt(self.output2.iface, 0x01, AUDIO, AUDIOSTREAMING, 0x00, None)?;
        writer.write(
            CS_INTERFACE,
            &[
                0x01, // bDescriptorSubtype = AS_GENERAL
                0x05, // bTerminalLink = output 1
                0x01, // bDelay
                0x01,
                0x00, // wFormatTag = PCM
            ],
        )?;
        writer.write(
            CS_INTERFACE,
            &[
                0x02, // bDescriptorSubtype = FORMAT_TYPE
                0x01, // bFormatType = FORMAT_TYPE_I
                0x02, // bNrChannels = 2
                0x02, // bSubFrameSize = 2 (16bit)
                0x10, // bBitResolution = 16
                0x01, // bSamFreqType
                0x80, //
                0xbb, //
                0x00, // tSamFreq = 48k = 0xbb80
            ],
        )?;
        writer.endpoint(&self.output2.ep)?;
        writer.write(
            CS_ENDPOINT,
            &[
                0x01, // bDescriptorSubtype = EP_GENERAL
                0x00, // bmAttributes
                0x00, // bLockDelayUnits
                0x00, //
                0x00, // wLockDelay
            ],
        )?;

        Ok(())
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        let req = xfer.request();
        if req.request_type == RequestType::Standard
            && req.recipient == Recipient::Interface
            && req.request == Request::GET_INTERFACE
            && req.length == 1
        {
            let iface = req.index as u8;

            if iface == self.input.iface.into() {
                xfer.accept_with(&[self.input.alt_setting]).ok();
            } else if iface == self.output1.iface.into() {
                xfer.accept_with(&[self.output1.alt_setting]).ok();
            } else if iface == self.output2.iface.into() {
                xfer.accept_with(&[self.output2.alt_setting]).ok();
            }
        }
    }

    fn control_out(&mut self, xfer: ControlOut<B>) {
        let req = xfer.request();
        if req.request_type == RequestType::Standard
            && req.recipient == Recipient::Interface
            && req.request == Request::SET_INTERFACE
        {
            let iface = req.index as u8;
            let alt_setting = req.value as u8;

            if iface == self.input.iface.into() {
                self.input.alt_setting = alt_setting;
                xfer.accept().ok();
            } else if iface == self.output1.iface.into() {
                self.output1.alt_setting = alt_setting;
                xfer.accept().ok();
            } else if iface == self.output2.iface.into() {
                self.output2.alt_setting = alt_setting;
                xfer.accept().ok();
            }
        }
    }
}
