//! Scull module in Rust.
use kernel::io_buffer::{IoBufferReader, IoBufferWriter};
use kernel::prelude::*;
use kernel::sync::{Arc, CondVar, UniqueArc, Mutex};
use kernel::user_ptr::UserSlicePtrWriter;
use kernel::{file, miscdev};
use kernel::file::{File, IoctlHandler};

module! {
    type: Scull,
    name: "scull",
    license: "GPL",
    params: {
        nr_devs: u32 {
            default: 4,
            permissions: 0o644,
            description: "Number of scull devices",
        },
    },
}

struct ScullDataInner {
    data: Vec<u8>,
}

struct ScullData {
    cond: CondVar,
    number: u32,
    inner: Mutex<ScullDataInner>,
}

impl ScullData {
    fn try_new(dev_id: u32) -> Result<Arc<Self>> {
        let mut data = Pin::from(UniqueArc::try_new(Self {
            cond: unsafe { CondVar::new() },
            number: dev_id,
            inner: unsafe { Mutex::new(ScullDataInner { data: Vec::new() }) },
        })?);

        let pinned = unsafe { data.as_mut().map_unchecked_mut(|d| &mut d.cond) };
        kernel::condvar_init!(pinned, "ScullData::cond");

        let pinned = unsafe { data.as_mut().map_unchecked_mut(|d| &mut d.inner) };
        kernel::mutex_init!(pinned, "Sculldata::inner");

        Ok(data.into())
    }
}

struct Scull {
    _devs: Vec<Pin<Box<miscdev::Registration<Scull>>>>,
}

struct ScullIoctlHandler;

impl IoctlHandler for ScullIoctlHandler {
    type Target<'a> = ();

    fn read(
        _this: Self::Target<'_>,
        _file: &File,
        _cmd: u32,
        _writer: &mut UserSlicePtrWriter,
    ) -> Result<i32> {
        pr_info!("read ioctl\n");
        if let Err(e) = _writer.write_slice(b"h") {
            pr_info!("opps!!, {:?}\n", e);
            Ok(0)
        } else {
            pr_info!("ok\n");
            Ok(-1)
        }
    }
}

#[vtable]
impl file::Operations for Scull {
    type Data = Arc<ScullData>;
    type OpenData = Arc<ScullData>;

    fn open(context: &Self::OpenData, file: &file::File) -> Result<Self::Data> {
        pr_info!("File for device {} was opened\n", context.number);
        if file.flags() & file::flags::O_ACCMODE == file::flags::O_WRONLY {
            context.inner.lock().data.clear();
        }
        Ok(context.clone())
    }

    fn read(
        _data: <Self::Data as kernel::ForeignOwnable>::Borrowed<'_>,
        _file: &file::File,
        _writer: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("File for device {} was read\n", _data.number);
        let offset = _offset.try_into()?;
        let vec = _data.inner.lock();
        let len = core::cmp::min(_writer.len(), vec.data.len().saturating_sub(offset));
        _writer.write_slice(&vec.data[offset..][..len])?;
        Ok(len)
    }

    fn write(
        _data: <Self::Data as kernel::ForeignOwnable>::Borrowed<'_>,
        _file: &file::File,
        _reader: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("File for device {} was written\n", _data.number);
        let offset = _offset.try_into()?;
        let len = _reader.len();
        let new_len = len.checked_add(offset).ok_or(EINVAL)?;
        let mut vec = _data.inner.lock();
        if new_len > vec.data.len() {
            vec.data.try_resize(new_len, 0)?;
        }
        _reader.read_slice(&mut vec.data[offset..][..len])?;
        Ok(len)
    }

    fn ioctl(
        _data: <Self::Data as kernel::ForeignOwnable>::Borrowed<'_>,
        _file: &file::File,
        _cmd: &mut file::IoctlCommand,
    ) -> Result<i32> {
        // implement _IOR(type, nr, datatype), which return "receive from scull"
        _cmd.dispatch::<ScullIoctlHandler>((), _file)
    }
}

impl kernel::Module for Scull {
    fn init(_name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        let count = {
            let lock = module.kernel_param_lock();
            (*nr_devs.read(&lock)).try_into()?
        };
        pr_info!("Hello world, {} devices!\n", count);
        let mut devs = Vec::try_with_capacity(count)?;
        for i in 0..count {
            let data = ScullData::try_new(i as u32)?;
            let reg = miscdev::Registration::new_pinned(fmt!("scull{i}"), data)?;
            devs.try_push(reg)?;
        }
        Ok(Self { _devs: devs })
    }
}
