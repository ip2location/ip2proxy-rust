use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    net::{Ipv4Addr, Ipv6Addr},
    path::Path,
    result::Result,
    str::FromStr,
};

use memmap::Mmap;

use crate::{consts, error, record};

#[derive(Debug)]
pub struct DB {
    path: String,
    db_type: u8,
    db_column: u8,
    db_year: u8,
    db_month: u8,
    db_day: u8,
    ipv4_db_count: u32,
    ipv4_db_addr: u32,
    ipv6_db_count: u32,
    ipv6_db_addr: u32,
    ipv4_index_base_addr: u32,
    ipv6_index_base_addr: u32,
    product_code: u8,
    license_code: u8,
    database_size: u32,
    file: Option<File>,
    mmap: Option<Mmap>,
}

impl Drop for DB {
    fn drop(&mut self) {
        // The 'file' field will be dropped automatically by Rust.
        match &mut self.mmap {
            Some(mmap) => {
                drop(mmap);
                self.mmap = None;
            }
            None => {}
        }
    }
}

impl DB {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, error::Error> {
        //! Loads a IP2Proxy Database .bin file from path
        let mut obj = Self::empty();
        obj.path = path.as_ref().to_string_lossy().into();
        obj.open()?;
        obj.read_header()?;
        Ok(obj)
    }

    pub fn from_file_mmap<P: AsRef<Path>>(path: P) -> Result<Self, error::Error> {
        //! Loads a IP2Proxy Database .bin file from path using
        let mut obj = Self::empty();
        obj.path = path.as_ref().to_string_lossy().into();
        obj.mmap()?;
        obj.read_header()?;
        Ok(obj)
    }

    pub fn print_db_info(&self) {
        //! Prints the DB Information
        println!("Database Path: {}", self.path);
        println!(" |- Database Oackage Version: {}", self.db_type);
        println!(
            " |- Database Version: {}.{}.{}",
            self.db_year, self.db_month, self.db_day
        );
    }

    pub fn ip_lookup(&mut self, ip: &str) -> Result<serde_json::value::Value, error::Error> {
        //! Lookup for the given IPv4 or IPv6 and returns the Geo information
        if Self::is_ipv4(ip) {
            let ip_number = u32::from(Ipv4Addr::from_str(ip)?);
            let mut record = self.ipv4_lookup(ip_number)?;
            record.ip = ip.into();
            let json_value = serde_json::json!(&record);
            return Ok(json_value);
        } else if Self::is_ipv6(ip) {
            let ipv6 = Ipv6Addr::from_str(ip)?;
            let mut record = self.ipv6_lookup(ipv6)?;
            record.ip = ip.into();
            let json_value = serde_json::json!(&record);
            return Ok(json_value);
        }
        Err(error::Error::InvalidIP("ip address is invalid".into()))
    }

    fn empty() -> Self {
        Self {
            path: "".into(),
            db_type: 0,
            db_column: 0,
            db_year: 0,
            db_month: 0,
            db_day: 0,
            ipv4_db_count: 0,
            ipv4_db_addr: 0,
            ipv6_db_count: 0,
            ipv6_db_addr: 0,
            ipv4_index_base_addr: 0,
            ipv6_index_base_addr: 0,
            product_code: 0,
            license_code: 0,
            database_size: 0,
            file: None,
            mmap: None,
        }
    }

    fn open(&mut self) -> Result<(), error::Error> {
        match File::open(self.path.clone()) {
            Ok(f) => {
                self.file = Some(f);
                return Ok(());
            }
            Err(e) => {
                return Err(error::Error::IoError(format!(
                    "Error opening DB file: {}",
                    e
                )))
            }
        };
    }

    fn mmap(&mut self) -> Result<(), error::Error> {
        match File::open(self.path.clone()) {
            Ok(f) => {
                match unsafe { Mmap::map(&f) } {
                    Ok(mmap) => {
                        self.mmap = Some(mmap);
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(error::Error::IoError(format!(
                            "error while mmaping db file: {}",
                            e
                        )))
                    }
                };
            }
            Err(e) => {
                return Err(error::Error::IoError(format!(
                    "Error opening DB file: {}",
                    e
                )))
            }
        };
    }

    fn read_header(&mut self) -> Result<(), error::Error> {
        self.db_type = self.read_u8(1)?;
        self.db_column = self.read_u8(2)?;
        self.db_year = self.read_u8(3)?;
        println!("{}", self.db_year);
        self.db_month = self.read_u8(4)?;
        self.db_day = self.read_u8(5)?;
        self.ipv4_db_count = self.read_u32(6)?;
        self.ipv4_db_addr = self.read_u32(10)?;
        self.ipv6_db_count = self.read_u32(14)?;
        self.ipv6_db_addr = self.read_u32(18)?;
        self.ipv4_index_base_addr = self.read_u32(22)?;
        self.ipv6_index_base_addr = self.read_u32(26)?;
        self.product_code = self.read_u8(27)?;
        self.license_code = self.read_u8(28)?;
        self.database_size = self.read_u32(29)?;
        if self.product_code != 2 {
            if self.db_year > 20 && self.product_code != 0 {
                let msg = format!("Incorrect IP2Location BIN file format. Please make sure that you are using the latest IP2Location BIN file.");
                // return Err(Error::new(ErrorKind::InvalidData, msg));
                return Err(error::Error::GenericError(msg));
            }
        }
        Ok(())
    }

    fn ipv4_lookup(&mut self, mut ip_number: u32) -> Result<record::Record, error::Error> {
        if ip_number == u32::MAX {
            ip_number -= 1;
        }
        let mut low = 0;
        let mut high = self.ipv4_db_count;
        if self.ipv4_index_base_addr > 0 {
            let index = ((ip_number >> 16) << 3) + self.ipv4_index_base_addr;
            low = self.read_u32(index as u64)?;
            high = self.read_u32((index + 4) as u64)?;
        }
        while low <= high {
            let mid = (low + high) >> 1;
            let ip_from =
                self.read_u32((self.ipv4_db_addr + mid * (self.db_column as u32) * 4) as u64)?;
            let ip_to = self
                .read_u32((self.ipv4_db_addr + (mid + 1) * (self.db_column as u32) * 4) as u64)?;
            if (ip_number >= ip_from) && (ip_number < ip_to) {
                return self.read_record(self.ipv4_db_addr + mid * (self.db_column as u32) * 4);
            } else {
                if ip_number < ip_from {
                    high = mid - 1;
                } else {
                    low = mid + 1;
                }
            }
        }
        Err("no record found".into())
    }

    fn ipv6_lookup(&mut self, ipv6: Ipv6Addr) -> Result<record::Record, error::Error> {
        let mut low = 0;
        let mut high = self.ipv6_db_count;
        if self.ipv6_index_base_addr > 0 {
            let num = (ipv6.octets()[0] as u32) * 256 + (ipv6.octets()[1] as u32);
            let index = (num << 3) + self.ipv6_index_base_addr;
            low = self.read_u32(index as u64)?;
            high = self.read_u32((index + 4) as u64)?;
        }
        while low <= high {
            let mid = (low + high) >> 1;
            let ip_from = self
                .read_ipv6((self.ipv6_db_addr + mid * ((self.db_column as u32) * 4 + 12)) as u64)?;
            let ip_to = self.read_ipv6(
                (self.ipv6_db_addr + (mid + 1) * ((self.db_column as u32) * 4 + 12)) as u64,
            )?;
            if (ipv6 >= ip_from) && (ipv6 < ip_to) {
                return self.read_record(
                    self.ipv6_db_addr + mid * ((self.db_column as u32) * 4 + 12) + 12,
                );
            } else {
                if ipv6 < ip_from {
                    high = mid - 1;
                } else {
                    low = mid + 1;
                }
            }
        }
        Err("no record found".into())
    }

    fn read_record(&mut self, row_addr: u32) -> Result<record::Record, error::Error> {
        let mut result = record::Record::new_empty();

        if consts::COUNTRY_POSITION[self.db_type as usize] > 0 {
            let index = self.read_u32(
                (row_addr + 4 * (consts::COUNTRY_POSITION[self.db_type as usize] - 1)).into(),
            )?;
            result.country_short = self.read_str(index.into())?;
            result.country_long = self.read_str((index + 3).into())?;
        }

        if consts::REGION_POSITION[self.db_type as usize] > 0 {
            let index = self.read_u32(
                (row_addr + 4 * (consts::REGION_POSITION[self.db_type as usize] - 1)).into(),
            )?;
            result.region = self.read_str(index.into())?;
        }

        if consts::CITY_POSITION[self.db_type as usize] > 0 {
            let index = self.read_u32(
                (row_addr + 4 * (consts::CITY_POSITION[self.db_type as usize] - 1)).into(),
            )?;
            result.city = self.read_str(index.into())?;
        }

        if consts::ISP_POSITION[self.db_type as usize] > 0 {
            let index = self.read_u32(
                (row_addr + 4 * (consts::ISP_POSITION[self.db_type as usize] - 1)).into(),
            )?;
            result.isp = self.read_str(index.into())?;
        }

        if consts::PROXYTYPE_POSITION[self.db_type as usize] > 0 {
            let index = self.read_u32(
                (row_addr + 4 * (consts::PROXYTYPE_POSITION[self.db_type as usize] - 1)).into(),
            )?;
            result.proxy_type = self.read_str(index.into())?;
        }

        if consts::DOMAIN_POSITION[self.db_type as usize] > 0 {
            let index = self.read_u32(
                (row_addr + 4 * (consts::DOMAIN_POSITION[self.db_type as usize] - 1)).into(),
            )?;
            result.domain = self.read_str(index.into())?;
        }

        if consts::USAGETYPE_POSITION[self.db_type as usize] > 0 {
            let index = self.read_u32(
                (row_addr + 4 * (consts::USAGETYPE_POSITION[self.db_type as usize] - 1)).into(),
            )?;
            result.usage_type = self.read_str(index.into())?;
        }

        if consts::ASN_POSITION[self.db_type as usize] > 0 {
            let index = self.read_u32(
                (row_addr + 4 * (consts::ASN_POSITION[self.db_type as usize] - 1)).into(),
            )?;
            result.as_name = self.read_str(index.into())?;
        }

        if consts::AS_POSITION[self.db_type as usize] > 0 {
            let index = self.read_u32(
                (row_addr + 4 * (consts::AS_POSITION[self.db_type as usize] - 1)).into(),
            )?;
            result.asn = self.read_str(index.into())?;
        }

        if consts::LASTSEEN_POSITION[self.db_type as usize] > 0 {
            let index = self.read_u32(
                (row_addr + 4 * (consts::LASTSEEN_POSITION[self.db_type as usize] - 1)).into(),
            )?;
            result.last_seen = self.read_str(index.into())?;
        }

        if consts::THREAT_POSITION[self.db_type as usize] > 0 {
            let index = self.read_u32(
                (row_addr + 4 * (consts::THREAT_POSITION[self.db_type as usize] - 1)).into(),
            )?;
            result.threat = self.read_str(index.into())?;
        }

        if consts::PROVIDER_POSITION[self.db_type as usize] > 0 {
            let index = self.read_u32(
                (row_addr + 4 * (consts::PROVIDER_POSITION[self.db_type as usize] - 1))
                    .into(),
            )?;
            result.provider = self.read_str(index.into())?;
        }

        if self.db_type == 1 {
            if result.country_short == "-" {
                result.is_proxy = 0;
            } else if result.proxy_type == "DCH" || result.proxy_type == "SES" {
                result.is_proxy = 2;
            } else {
                result.is_proxy = 1;
            }
        } else {
            if result.proxy_type == "-" {
                result.is_proxy = 0;
        } else if result.proxy_type == "DCH" || result.proxy_type == "SES" {
                result.is_proxy = 2;
            } else {
                result.is_proxy = 1;
            }
        }

        Ok(result)
    }

    fn read_u8(&mut self, offset: u64) -> Result<u8, error::Error> {
        if self.file.is_some() {
            let mut f = self.file.as_ref().unwrap();
            f.seek(SeekFrom::Start(offset - 1))?;
            let mut buf = [0_u8; 1];
            f.read(&mut buf)?;
            Ok(buf[0])
        } else if self.mmap.is_some() {
            let m = self.mmap.as_ref().unwrap();
            Ok(m[(offset - 1) as usize])
        } else {
            Err(error::Error::InvalidState("db is not open".into()))
        }
    }

    fn read_u32(&mut self, offset: u64) -> Result<u32, error::Error> {
        if self.file.is_some() {
            let mut f = self.file.as_ref().unwrap();
            f.seek(SeekFrom::Start(offset - 1))?;
            let mut buf = [0_u8; 4];
            f.read(&mut buf)?;
            let result = u32::from_ne_bytes(buf);
            Ok(result)
        } else if self.mmap.is_some() {
            let m = self.mmap.as_ref().unwrap();
            let mut buf = [0_u8; 4];
            buf[0] = m[(offset - 1) as usize];
            buf[1] = m[offset as usize];
            buf[2] = m[(offset + 1) as usize];
            buf[3] = m[(offset + 2) as usize];
            let result = u32::from_ne_bytes(buf);
            Ok(result)
        } else {
            Err(error::Error::InvalidState("db is not open".into()))
        }
    }

    #[allow(dead_code)]
    fn read_f32(&mut self, offset: u64) -> Result<f32, error::Error> {
        if self.file.is_some() {
            let mut f = self.file.as_ref().unwrap();
            f.seek(SeekFrom::Start(offset - 1))?;
            let mut buf = [0_u8; 4];
            f.read(&mut buf)?;
            let result = f32::from_ne_bytes(buf);
            Ok(result)
        } else if self.mmap.is_some() {
            let m = self.mmap.as_ref().unwrap();
            let mut buf = [0_u8; 4];
            buf[0] = m[(offset - 1) as usize];
            buf[1] = m[offset as usize];
            buf[2] = m[(offset + 1) as usize];
            buf[3] = m[(offset + 2) as usize];
            let result = f32::from_ne_bytes(buf);
            Ok(result)
        } else {
            Err(error::Error::InvalidState("db is not open".into()))
        }
    }

    fn read_str(&mut self, offset: u64) -> Result<String, error::Error> {
        if self.file.is_some() {
            let len = self.read_u8(offset + 1)? as usize;
            let mut f = self.file.as_ref().unwrap();
            f.seek(SeekFrom::Start(offset + 1))?;
            let mut buf = vec![0_u8; len];
            f.read(&mut buf)?;
            let s = std::str::from_utf8(&buf)?;
            Ok(s.into())
        } else if self.mmap.is_some() {
            let len = self.read_u8(offset + 1)? as usize;
            let m = self.mmap.as_ref().unwrap();
            let mut buf = vec![0_u8; len];
            for i in 0..len {
                buf[i] = m[(offset + 1) as usize + i];
            }
            let s = std::str::from_utf8(&buf)?;
            Ok(s.into())
        } else {
            Err(error::Error::InvalidState("db is not open".into()))
        }
    }

    fn read_ipv6(&mut self, offset: u64) -> Result<Ipv6Addr, error::Error> {
        let mut buf = [0_u8; 16];
        let mut i = 0;
        let mut j = 15;
        while i < 16 {
            buf[i] = self.read_u8(offset + j)?;
            i += 1;
            if j > 0 {
                j -= 1;
            }
        }
        let result = Ipv6Addr::from(buf);
        Ok(result)
    }

    fn is_ipv4(ip: &str) -> bool {
        match Ipv4Addr::from_str(ip) {
            Ok(_) => {
                true
            }
            Err(_) => {
                false
            }
        }
    }

    fn is_ipv6(ip: &str) -> bool {
        match Ipv6Addr::from_str(ip) {
            Ok(_) => {
                true
            }
            Err(_) => {
                false
            }
        }
    }
}
