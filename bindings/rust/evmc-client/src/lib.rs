// Copyright (C) 2020 Second State.
// This file is part of EVMC-Client.

// EVMC-Client is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// EVMC-Client is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#[macro_use]
extern crate enum_primitive;
pub mod host;
mod loader;
pub mod types;
pub use crate::loader::{load_and_create, EvmcLoaderErrorCode};
use crate::types::*;
use evmc_sys as ffi;
use std::ffi::CStr;

extern "C" {
    fn evmc_create() -> *mut ffi::evmc_instance;
}

pub struct EvmcVm {
    handle: *mut ffi::evmc_instance,
    host_interface: *mut ffi::evmc_host_interface,
}

impl EvmcVm {
    pub fn get_abi_version(&self) -> i32 {
        unsafe {
            let version: i32 = (*self.handle).abi_version;
            version
        }
    }

    pub fn get_name(&self) -> &str {
        unsafe {
            let c_str: &CStr = CStr::from_ptr((*self.handle).name);
            c_str.to_str().unwrap()
        }
    }

    pub fn get_version(&self) -> &str {
        unsafe {
            let c_str: &CStr = CStr::from_ptr((*self.handle).version);
            c_str.to_str().unwrap()
        }
    }

    pub fn destroy(&self) {
        unsafe { ((*self.handle).destroy.unwrap())(self.handle) }
    }

    pub fn execute(
        &self,
        ctx: &mut dyn host::HostContext,
        rev: Revision,
        kind: CallKind,
        is_static: bool,
        depth: i32,
        gas: i64,
        destination: &Address,
        sender: &Address,
        input: &Bytes,
        value: &Bytes32,
        code: &Bytes,
        create2_salt: &Bytes32,
    ) -> (&Bytes, i64, StatusCode) {
        let mut ext_ctx = host::ExtendedContext {
            context: ffi::evmc_context {
                host: self.host_interface,
            },
            hctx: ctx,
        };
        let mut evmc_flags: u32 = 0;
        unsafe {
            if is_static {
                evmc_flags |=
                    std::mem::transmute::<ffi::evmc_flags, u32>(ffi::evmc_flags::EVMC_STATIC);
            }
        }
        let evmc_message = Box::into_raw(Box::new({
            ffi::evmc_message {
                kind: kind,
                flags: evmc_flags,
                depth: depth,
                gas: gas,
                destination: ffi::evmc_address {
                    bytes: *destination,
                },
                sender: ffi::evmc_address { bytes: *sender },
                input_data: input.as_ptr(),
                input_size: input.len(),
                value: ffi::evmc_uint256be { bytes: *value },
                create2_salt: ffi::evmc_bytes32 {
                    bytes: *create2_salt,
                },
            }
        }));
        unsafe {
            let result = ((*self.handle).execute.unwrap())(
                self.handle,
                &mut ext_ctx.context,
                rev,
                evmc_message,
                code.as_ptr(),
                code.len(),
            );
            return (
                std::slice::from_raw_parts(result.output_data, result.output_size),
                result.gas_left,
                result.status_code,
            );
        }
    }

    pub fn has_capability(&self, capability: Capabilities) -> bool {
        unsafe {
            std::mem::transmute::<Capabilities, u32>(capability)
                == ((*self.handle).get_capabilities.unwrap())(self.handle)
        }
    }
}

pub fn load(fname: &str) -> (EvmcVm, Result<EvmcLoaderErrorCode, &'static str>) {
    let (instance, ec) = load_and_create(fname);
    (
        EvmcVm {
            handle: instance,
            host_interface: Box::into_raw(Box::new(host::get_evmc_host_interface())),
        },
        ec,
    )
}

pub fn create() -> EvmcVm {
    unsafe {
        EvmcVm {
            handle: evmc_create(),
            host_interface: Box::into_raw(Box::new(host::get_evmc_host_interface())),
        }
    }
}
