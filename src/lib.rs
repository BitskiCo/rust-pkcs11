// Copyright 2017 Marcus Heese
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#![allow(non_camel_case_types, non_snake_case)]

extern crate libloading;
extern crate num_bigint;

#[cfg(test)]
mod tests;

/// This module is basically a full conversion of the `pkcs11t.h` C header file.
pub mod types;
/// This module is basically a full conversion of the `pkcs11f.h` C header file.
pub mod functions;
/// The error types are defined here - they are used throughout the crate.
pub mod errors;

use types::*;
use functions::*;
use errors::Error;


use std::mem;
use std::ptr;
use std::ffi::CString;
//use libc::c_uchar;


trait CkFrom<T> {
  fn from(T) -> Self;
}

impl CkFrom<bool> for CK_BBOOL {
  fn from(b: bool) -> Self {
    match b {
      true => 1,
      false => 0,
    }
  }
}

impl CkFrom<CK_BBOOL> for bool {
  fn from(b: CK_BBOOL) -> bool {
    match b {
      0 => false,
      _ => true,
    }
  }
}

fn label_from_str(label: &str) -> [CK_UTF8CHAR; 32] {
  // initialize a fixed-size array with whitespace characters
  let mut lab: [CK_UTF8CHAR; 32] = [32; 32];
  let mut i = 0;
  for c in label.chars() {
    if i + c.len_utf8() <= 32 {
      let mut buf = [0; 4];
      let bytes = c.encode_utf8(&mut buf).as_bytes();
      for b in bytes {
        lab[i] = b.clone();
        i += 1;
      }
    } else {
      break;
    }
  }
  lab
}

#[derive(Debug)]
pub struct Ctx {
  lib: libloading::Library,
  _is_initialized: bool,
  C_Initialize: C_Initialize,
  C_Finalize: C_Finalize,
  C_GetInfo: C_GetInfo,
  C_GetFunctionList: C_GetFunctionList,
  C_GetSlotList: C_GetSlotList,
  C_GetSlotInfo: C_GetSlotInfo,
  C_GetTokenInfo: C_GetTokenInfo,
  C_GetMechanismList: C_GetMechanismList,
  C_GetMechanismInfo: C_GetMechanismInfo,
  C_InitToken: C_InitToken,
  C_InitPIN: C_InitPIN,
  C_SetPIN: C_SetPIN,
  C_OpenSession: C_OpenSession,
  C_CloseSession: C_CloseSession,
  C_CloseAllSessions: C_CloseAllSessions,
  C_GetSessionInfo: C_GetSessionInfo,
  C_GetOperationState: C_GetOperationState,
  C_SetOperationState: C_SetOperationState,
  C_Login: C_Login,
  C_Logout: C_Logout,
  C_CreateObject: C_CreateObject,
  C_CopyObject: C_CopyObject,
  C_DestroyObject: C_DestroyObject,
  C_GetObjectSize: C_GetObjectSize,
  C_GetAttributeValue: C_GetAttributeValue,
  C_SetAttributeValue: C_SetAttributeValue,
  C_FindObjectsInit: C_FindObjectsInit,
  C_FindObjects: C_FindObjects,
  C_FindObjectsFinal: C_FindObjectsFinal,
  C_EncryptInit: C_EncryptInit,
  C_Encrypt: C_Encrypt,
  C_EncryptUpdate: C_EncryptUpdate,
  C_EncryptFinal: C_EncryptFinal,
  C_DecryptInit: C_DecryptInit,
  C_Decrypt: C_Decrypt,
  C_DecryptUpdate: C_DecryptUpdate,
  C_DecryptFinal: C_DecryptFinal,
  C_DigestInit: C_DigestInit,
  C_Digest: C_Digest,
  C_DigestUpdate: C_DigestUpdate,
  C_DigestKey: C_DigestKey,
  C_DigestFinal: C_DigestFinal,
  C_SignInit: C_SignInit,
  C_Sign: C_Sign,
  C_SignUpdate: C_SignUpdate,
  C_SignFinal: C_SignFinal,
  C_SignRecoverInit: C_SignRecoverInit,
  C_SignRecover: C_SignRecover,
  C_VerifyInit: C_VerifyInit,
  C_Verify: C_Verify,
  C_VerifyUpdate: C_VerifyUpdate,
  C_VerifyFinal: C_VerifyFinal,
  C_VerifyRecoverInit: C_VerifyRecoverInit,
  C_VerifyRecover: C_VerifyRecover,
  C_DigestEncryptUpdate: C_DigestEncryptUpdate,
  C_DecryptDigestUpdate: C_DecryptDigestUpdate,
  C_SignEncryptUpdate: C_SignEncryptUpdate,
  C_DecryptVerifyUpdate: C_DecryptVerifyUpdate,
  C_GenerateKey: C_GenerateKey,
  C_GenerateKeyPair: C_GenerateKeyPair,
  C_WrapKey: C_WrapKey,
  C_UnwrapKey: C_UnwrapKey,
  C_DeriveKey: C_DeriveKey,
  C_SeedRandom: C_SeedRandom,
  C_GenerateRandom: C_GenerateRandom,
  C_GetFunctionStatus: C_GetFunctionStatus,
  C_CancelFunction: C_CancelFunction,
  C_WaitForSlotEvent: C_WaitForSlotEvent,
}

impl Ctx {
  pub fn new(filename: &'static str) -> Result<Ctx, Error> {
    unsafe {
      let lib = libloading::Library::new(filename)?;
      let mut list: CK_FUNCTION_LIST_PTR = mem::uninitialized();
      {
        let func: libloading::Symbol<unsafe extern "C" fn(CK_FUNCTION_LIST_PTR_PTR) -> CK_RV> = lib.get(b"C_GetFunctionList")?;
        match func(&mut list) {
          CKR_OK => (),
          err => return Err(Error::Pkcs11(err)),
        }
      }

      Ok(Ctx {
        lib: lib,
        _is_initialized: false,
        C_Initialize: (*list).C_Initialize.ok_or(Error::Module("C_Initialize function not found"))?,
        C_Finalize: (*list).C_Finalize.ok_or(Error::Module("C_Finalize function not found"))?,
        C_GetInfo: (*list).C_GetInfo.ok_or(Error::Module("C_GetInfo function not found"))?,
        C_GetFunctionList: (*list).C_GetFunctionList.ok_or(Error::Module("C_GetFunctionList function not found"))?,
        C_GetSlotList: (*list).C_GetSlotList.ok_or(Error::Module("C_GetSlotList function not found"))?,
        C_GetSlotInfo: (*list).C_GetSlotInfo.ok_or(Error::Module("C_GetSlotInfo function not found"))?,
        C_GetTokenInfo: (*list).C_GetTokenInfo.ok_or(Error::Module("C_GetTokenInfo function not found"))?,
        C_GetMechanismList: (*list).C_GetMechanismList.ok_or(Error::Module("C_GetMechanismList function not found"))?,
        C_GetMechanismInfo: (*list).C_GetMechanismInfo.ok_or(Error::Module("C_GetMechanismInfo function not found"))?,
        C_InitToken: (*list).C_InitToken.ok_or(Error::Module("C_InitToken function not found"))?,
        C_InitPIN: (*list).C_InitPIN.ok_or(Error::Module("C_InitPIN function not found"))?,
        C_SetPIN: (*list).C_SetPIN.ok_or(Error::Module("C_SetPIN function not found"))?,
        C_OpenSession: (*list).C_OpenSession.ok_or(Error::Module("C_OpenSession function not found"))?,
        C_CloseSession: (*list).C_CloseSession.ok_or(Error::Module("C_CloseSession function not found"))?,
        C_CloseAllSessions: (*list).C_CloseAllSessions.ok_or(Error::Module("C_CloseAllSessions function not found"))?,
        C_GetSessionInfo: (*list).C_GetSessionInfo.ok_or(Error::Module("C_GetSessionInfo function not found"))?,
        C_GetOperationState: (*list).C_GetOperationState.ok_or(Error::Module("C_GetOperationState function not found"))?,
        C_SetOperationState: (*list).C_SetOperationState.ok_or(Error::Module("C_SetOperationState function not found"))?,
        C_Login: (*list).C_Login.ok_or(Error::Module("C_Login function not found"))?,
        C_Logout: (*list).C_Logout.ok_or(Error::Module("C_Logout function not found"))?,
        C_CreateObject: (*list).C_CreateObject.ok_or(Error::Module("C_CreateObject function not found"))?,
        C_CopyObject: (*list).C_CopyObject.ok_or(Error::Module("C_CopyObject function not found"))?,
        C_DestroyObject: (*list).C_DestroyObject.ok_or(Error::Module("C_DestroyObject function not found"))?,
        C_GetObjectSize: (*list).C_GetObjectSize.ok_or(Error::Module("C_GetObjectSize function not found"))?,
        C_GetAttributeValue: (*list).C_GetAttributeValue.ok_or(Error::Module("C_GetAttributeValue function not found"))?,
        C_SetAttributeValue: (*list).C_SetAttributeValue.ok_or(Error::Module("C_SetAttributeValue function not found"))?,
        C_FindObjectsInit: (*list).C_FindObjectsInit.ok_or(Error::Module("C_FindObjectsInit function not found"))?,
        C_FindObjects: (*list).C_FindObjects.ok_or(Error::Module("C_FindObjects function not found"))?,
        C_FindObjectsFinal: (*list).C_FindObjectsFinal.ok_or(Error::Module("C_FindObjectsFinal function not found"))?,
        C_EncryptInit: (*list).C_EncryptInit.ok_or(Error::Module("C_EncryptInit function not found"))?,
        C_Encrypt: (*list).C_Encrypt.ok_or(Error::Module("C_Encrypt function not found"))?,
        C_EncryptUpdate: (*list).C_EncryptUpdate.ok_or(Error::Module("C_EncryptUpdate function not found"))?,
        C_EncryptFinal: (*list).C_EncryptFinal.ok_or(Error::Module("C_EncryptFinal function not found"))?,
        C_DecryptInit: (*list).C_DecryptInit.ok_or(Error::Module("C_DecryptInit function not found"))?,
        C_Decrypt: (*list).C_Decrypt.ok_or(Error::Module("C_Decrypt function not found"))?,
        C_DecryptUpdate: (*list).C_DecryptUpdate.ok_or(Error::Module("C_DecryptUpdate function not found"))?,
        C_DecryptFinal: (*list).C_DecryptFinal.ok_or(Error::Module("C_DecryptFinal function not found"))?,
        C_DigestInit: (*list).C_DigestInit.ok_or(Error::Module("C_DigestInit function not found"))?,
        C_Digest: (*list).C_Digest.ok_or(Error::Module("C_Digest function not found"))?,
        C_DigestUpdate: (*list).C_DigestUpdate.ok_or(Error::Module("C_DigestUpdate function not found"))?,
        C_DigestKey: (*list).C_DigestKey.ok_or(Error::Module("C_DigestKey function not found"))?,
        C_DigestFinal: (*list).C_DigestFinal.ok_or(Error::Module("C_DigestFinal function not found"))?,
        C_SignInit: (*list).C_SignInit.ok_or(Error::Module("C_SignInit function not found"))?,
        C_Sign: (*list).C_Sign.ok_or(Error::Module("C_Sign function not found"))?,
        C_SignUpdate: (*list).C_SignUpdate.ok_or(Error::Module("C_SignUpdate function not found"))?,
        C_SignFinal: (*list).C_SignFinal.ok_or(Error::Module("C_SignFinal function not found"))?,
        C_SignRecoverInit: (*list).C_SignRecoverInit.ok_or(Error::Module("C_SignRecoverInit function not found"))?,
        C_SignRecover: (*list).C_SignRecover.ok_or(Error::Module("C_SignRecover function not found"))?,
        C_VerifyInit: (*list).C_VerifyInit.ok_or(Error::Module("C_VerifyInit function not found"))?,
        C_Verify: (*list).C_Verify.ok_or(Error::Module("C_Verify function not found"))?,
        C_VerifyUpdate: (*list).C_VerifyUpdate.ok_or(Error::Module("C_VerifyUpdate function not found"))?,
        C_VerifyFinal: (*list).C_VerifyFinal.ok_or(Error::Module("C_VerifyFinal function not found"))?,
        C_VerifyRecoverInit: (*list).C_VerifyRecoverInit.ok_or(Error::Module("C_VerifyRecoverInit function not found"))?,
        C_VerifyRecover: (*list).C_VerifyRecover.ok_or(Error::Module("C_VerifyRecover function not found"))?,
        C_DigestEncryptUpdate: (*list).C_DigestEncryptUpdate.ok_or(Error::Module("C_DigestEncryptUpdate function not found"))?,
        C_DecryptDigestUpdate: (*list).C_DecryptDigestUpdate.ok_or(Error::Module("C_DecryptDigestUpdate function not found"))?,
        C_SignEncryptUpdate: (*list).C_SignEncryptUpdate.ok_or(Error::Module("C_SignEncryptUpdate function not found"))?,
        C_DecryptVerifyUpdate: (*list).C_DecryptVerifyUpdate.ok_or(Error::Module("C_DecryptVerifyUpdate function not found"))?,
        C_GenerateKey: (*list).C_GenerateKey.ok_or(Error::Module("C_GenerateKey function not found"))?,
        C_GenerateKeyPair: (*list).C_GenerateKeyPair.ok_or(Error::Module("C_GenerateKeyPair function not found"))?,
        C_WrapKey: (*list).C_WrapKey.ok_or(Error::Module("C_WrapKey function not found"))?,
        C_UnwrapKey: (*list).C_UnwrapKey.ok_or(Error::Module("C_UnwrapKey function not found"))?,
        C_DeriveKey: (*list).C_DeriveKey.ok_or(Error::Module("C_DeriveKey function not found"))?,
        C_SeedRandom: (*list).C_SeedRandom.ok_or(Error::Module("C_SeedRandom function not found"))?,
        C_GenerateRandom: (*list).C_GenerateRandom.ok_or(Error::Module("C_GenerateRandom function not found"))?,
        C_GetFunctionStatus: (*list).C_GetFunctionStatus.ok_or(Error::Module("C_GetFunctionStatus function not found"))?,
        C_CancelFunction: (*list).C_CancelFunction.ok_or(Error::Module("C_CancelFunction function not found"))?,
        C_WaitForSlotEvent: (*list).C_WaitForSlotEvent.ok_or(Error::Module("C_WaitForSlotEvent function not found"))?,
      })
    }
  }

  pub fn new_and_initialize(filename: &'static str) -> Result<Ctx, Error> {
    let mut ctx = Ctx::new(filename)?;
    ctx.initialize(None)?;
    Ok(ctx)
  }

  pub fn is_initialized(&self) -> bool {
    self._is_initialized
  }

  fn initialized(&self) -> Result<(), Error> {
    if !self._is_initialized {
      Err(Error::Module("module not initialized"))
    } else {
      Ok(())
    }
  }

  fn not_initialized(&self) -> Result<(), Error> {
    if self._is_initialized {
      Err(Error::Module("module already initialized"))
    } else {
      Ok(())
    }
  }

  pub fn initialize(&mut self, init_args: Option<CK_C_INITIALIZE_ARGS>) -> Result<(), Error> {
    self.not_initialized()?;
    match (self.C_Initialize)(&init_args.unwrap_or(CK_C_INITIALIZE_ARGS::new())) {
      CKR_OK => {
        self._is_initialized = true;
        Ok(())
      }
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn finalize(&mut self) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_Finalize)(ptr::null()) {
      CKR_OK => {
        self._is_initialized = false;
        Ok(())
      }
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn get_info(&self) -> Result<CK_INFO, Error> {
    self.initialized()?;
    let info = CK_INFO::new();
    match (self.C_GetInfo)(&info) {
      CKR_OK => Ok(info),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn get_function_list(&self) -> Result<CK_FUNCTION_LIST, Error> {
    let list: CK_FUNCTION_LIST_PTR = unsafe { mem::uninitialized() };
    match (self.C_GetFunctionList)(&list) {
      CKR_OK => unsafe { Ok((*list).clone()) },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn get_slot_list(&self, token_present: bool) -> Result<Vec<CK_SLOT_ID>, Error> {
    self.initialized()?;
    let mut slots_len: CK_ULONG = 0;
    match (self.C_GetSlotList)(CkFrom::from(token_present), ptr::null(), &mut slots_len) {
      CKR_OK => {
        // now slots_len contains the number of slots,
        // and we can generate a vector with the right capacity
        // important is to pass slots_len **again** because in
        // the 2nd call it is used to tell C how big the memory
        // in slots is.
        let mut slots = Vec::<CK_SLOT_ID>::with_capacity(slots_len);
        let slots_ptr = slots.as_mut_ptr();
        match (self.C_GetSlotList)(CkFrom::from(token_present), slots_ptr, &slots_len) {
          CKR_OK => {
            unsafe {
              slots.set_len(slots_len);
            }
            Ok(slots)
          }
          err => Err(Error::Pkcs11(err)),
        }
      }
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn get_slot_info(&self, slot_id: CK_SLOT_ID) -> Result<CK_SLOT_INFO, Error> {
    self.initialized()?;
    let info: CK_SLOT_INFO = Default::default();
    match (self.C_GetSlotInfo)(slot_id, &info) {
      CKR_OK => Ok(info),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn get_token_info(&self, slot_id: CK_SLOT_ID) -> Result<CK_TOKEN_INFO, Error> {
    self.initialized()?;
    let info: CK_TOKEN_INFO = Default::default();
    match (self.C_GetTokenInfo)(slot_id, &info) {
      CKR_OK => Ok(info),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn get_mechanism_list(&self, slot_id: CK_SLOT_ID) -> Result<Vec<CK_MECHANISM_TYPE>, Error> {
    self.initialized()?;
    let mut count: CK_ULONG = 0;
    match (self.C_GetMechanismList)(slot_id, ptr::null(), &mut count) {
      CKR_OK => {
        // see get_slot_list() for an explanation - it works the same way
        let mut list = Vec::<CK_MECHANISM_TYPE>::with_capacity(count);
        let list_ptr = list.as_mut_ptr();
        match (self.C_GetMechanismList)(slot_id, list_ptr, &count) {
          CKR_OK => {
            unsafe {
              list.set_len(count);
            }
            Ok(list)
          }
          err => Err(Error::Pkcs11(err)),
        }
      }
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn get_mechanism_info(&self, slot_id: CK_SLOT_ID, mechanism_type: CK_MECHANISM_TYPE) -> Result<CK_MECHANISM_INFO, Error> {
    self.initialized()?;
    let info: CK_MECHANISM_INFO = Default::default();
    match (self.C_GetMechanismInfo)(slot_id, mechanism_type, &info) {
      CKR_OK => Ok(info),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn init_token<'a, 'b>(&self, slot_id: CK_SLOT_ID, pin: Option<&'a str>, label: &'b str) -> Result<(), Error> {
    self.initialized()?;
    let formatted_label = label_from_str(label).to_vec().as_ptr();
    match pin {
      Some(pin) => if let Ok(cpin) = CString::new(pin) {
        let cpin_bytes = cpin.into_bytes();
        match (self.C_InitToken)(slot_id, cpin_bytes.as_ptr(), cpin_bytes.len(), formatted_label) {
          CKR_OK => Ok(()),
          err => Err(Error::Pkcs11(err)),
        }
      } else {
        Err(Error::InvalidInput("PIN contains a nul byte"))
      },
      None => {
        // CKF_PROTECTED_AUTHENTICATION_PATH requires a NULL pointer
        match (self.C_InitToken)(slot_id, ptr::null(), 0, formatted_label) {
          CKR_OK => Ok(()),
          err => Err(Error::Pkcs11(err)),
        }
      }
    }
  }

  pub fn init_pin<'a>(&self, session: CK_SESSION_HANDLE, pin: Option<&'a str>) -> Result<(), Error> {
    self.initialized()?;
    match pin {
      Some(pin) => if let Ok(cpin) = CString::new(pin) {
        let cpin_bytes = cpin.into_bytes();
        match (self.C_InitPIN)(session, cpin_bytes.as_ptr(), cpin_bytes.len()) {
          CKR_OK => Ok(()),
          err => Err(Error::Pkcs11(err)),
        }
      } else {
        Err(Error::InvalidInput("PIN contains a nul byte"))
      },
      None => match (self.C_InitPIN)(session, ptr::null(), 0) {
        CKR_OK => Ok(()),
        err => Err(Error::Pkcs11(err)),
      },
    }
  }

  pub fn set_pin<'a, 'b>(&self, session: CK_SESSION_HANDLE, old_pin: Option<&'a str>, new_pin: Option<&'b str>) -> Result<(), Error> {
    self.initialized()?;
    if old_pin.is_none() && new_pin.is_none() {
      match (self.C_SetPIN)(session, ptr::null(), 0, ptr::null(), 0) {
        CKR_OK => Ok(()),
        err => Err(Error::Pkcs11(err)),
      }
    } else if old_pin.is_some() && new_pin.is_some() {
      let old_cpin_res = CString::new(old_pin.unwrap());
      let new_cpin_res = CString::new(new_pin.unwrap());
      if old_cpin_res.is_err() {
        return Err(Error::InvalidInput("Old PIN contains a nul byte"));
      }
      if new_cpin_res.is_err() {
        return Err(Error::InvalidInput("New PIN contains a nul byte"));
      }
      let old_cpin = old_cpin_res.unwrap().into_bytes();
      let new_cpin = new_cpin_res.unwrap().into_bytes();
      match (self.C_SetPIN)(session, old_cpin.as_ptr(), old_cpin.len(), new_cpin.as_ptr(), new_cpin.len()) {
        CKR_OK => Ok(()),
        err => Err(Error::Pkcs11(err)),
      }
    } else {
      Err(Error::InvalidInput("both PINs must be either set or unset"))
    }
  }

  pub fn open_session(&self, slot_id: CK_SLOT_ID, flags: CK_FLAGS, application: Option<CK_VOID_PTR>, notify: CK_NOTIFY) -> Result<CK_SESSION_HANDLE, Error> {
    self.initialized()?;
    let mut session: CK_SESSION_HANDLE = 0;
    match (self.C_OpenSession)(slot_id, flags, application.unwrap_or(ptr::null()), notify, &mut session) {
      CKR_OK => Ok(session),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn close_session(&self, session: CK_SESSION_HANDLE) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_CloseSession)(session) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn close_all_sessions(&self, slot_id: CK_SLOT_ID) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_CloseAllSessions)(slot_id) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn get_session_info(&self, session: CK_SESSION_HANDLE) -> Result<CK_SESSION_INFO, Error> {
    self.initialized()?;
    let info: CK_SESSION_INFO = Default::default();
    match (self.C_GetSessionInfo)(session, &info) {
      CKR_OK => Ok(info),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn get_operation_state(&self, session: CK_SESSION_HANDLE) -> Result<Vec<CK_BYTE>, Error> {
    self.initialized()?;
    let mut state_length: CK_ULONG = 0;
    match (self.C_GetOperationState)(session, ptr::null(), &mut state_length) {
      CKR_OK => {
        let mut state: Vec<CK_BYTE> = Vec::with_capacity(state_length);
        let state_ptr = state.as_mut_ptr();
        match (self.C_GetOperationState)(session, state_ptr, &state_length) {
          CKR_OK => {
            unsafe {
              state.set_len(state_length);
            }
            Ok(state)
          }
          err => Err(Error::Pkcs11(err)),
        }
      }
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn set_operation_state(
    &self,
    session: CK_SESSION_HANDLE,
    operation_state: Vec<CK_BYTE>,
    encryption_key: Option<CK_OBJECT_HANDLE>,
    authentication_key: Option<CK_OBJECT_HANDLE>,
  ) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_SetOperationState)(session, operation_state.as_ptr(), operation_state.len(), encryption_key.unwrap_or(0), authentication_key.unwrap_or(0)) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn login<'a>(&self, session: CK_SESSION_HANDLE, user_type: CK_USER_TYPE, pin: Option<&'a str>) -> Result<(), Error> {
    self.initialized()?;
    match pin {
      Some(pin) => if let Ok(cpin) = CString::new(pin) {
        let cpin_bytes = cpin.into_bytes();
        match (self.C_Login)(session, user_type, cpin_bytes.as_ptr(), cpin_bytes.len()) {
          CKR_OK => Ok(()),
          err => Err(Error::Pkcs11(err)),
        }
      } else {
        Err(Error::InvalidInput("PIN contains a nul byte"))
      },
      None => match (self.C_Login)(session, user_type, ptr::null(), 0) {
        CKR_OK => Ok(()),
        err => Err(Error::Pkcs11(err)),
      },
    }
  }

  pub fn logout(&self, session: CK_SESSION_HANDLE) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_Logout)(session) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn create_object(&self, session: CK_SESSION_HANDLE, template: &Vec<CK_ATTRIBUTE>) -> Result<CK_OBJECT_HANDLE, Error> {
    self.initialized()?;
    let mut oh: CK_OBJECT_HANDLE = CK_INVALID_HANDLE;
    match (self.C_CreateObject)(session, template.as_slice().as_ptr(), template.len(), &mut oh) {
      CKR_OK => Ok(oh),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn copy_object(&self, session: CK_SESSION_HANDLE, object: CK_OBJECT_HANDLE, template: &Vec<CK_ATTRIBUTE>) -> Result<CK_OBJECT_HANDLE, Error> {
    self.initialized()?;
    let mut oh: CK_OBJECT_HANDLE = CK_INVALID_HANDLE;
    match (self.C_CopyObject)(session, object, template.as_slice().as_ptr(), template.len(), &mut oh) {
      CKR_OK => Ok(oh),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn destroy_object(&self, session: CK_SESSION_HANDLE, object: CK_OBJECT_HANDLE) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_DestroyObject)(session, object) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn get_object_size(&self, session: CK_SESSION_HANDLE, object: CK_OBJECT_HANDLE) -> Result<CK_ULONG, Error> {
    self.initialized()?;
    let mut size: CK_ULONG = 0;
    match (self.C_GetObjectSize)(session, object, &mut size) {
      CKR_OK => Ok(size),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn get_attribute_value<'a>(&self, session: CK_SESSION_HANDLE, object: CK_OBJECT_HANDLE, template: &'a Vec<CK_ATTRIBUTE>) -> Result<(CK_RV, &'a Vec<CK_ATTRIBUTE>), Error> {
    self.initialized()?;
    /*
      Note that the error codes CKR_ATTRIBUTE_SENSITIVE, CKR_ATTRIBUTE_TYPE_INVALID, and CKR_BUFFER_TOO_SMALL
      do not denote true errors for C_GetAttributeValue.  If a call to C_GetAttributeValue returns any of these three
      values, then the call MUST nonetheless have processed every attribute in the template supplied to
      C_GetAttributeValue.  Each attribute in the template whose value can be returned by the call to
      C_GetAttributeValue will be returned by the call to C_GetAttributeValue.
    */
    match (self.C_GetAttributeValue)(session, object, template.as_slice().as_ptr(), template.len()) {
      CKR_OK => Ok((CKR_OK, template)),
      CKR_ATTRIBUTE_SENSITIVE => Ok((CKR_ATTRIBUTE_SENSITIVE, template)),
      CKR_ATTRIBUTE_TYPE_INVALID => Ok((CKR_ATTRIBUTE_TYPE_INVALID, template)),
      CKR_BUFFER_TOO_SMALL => Ok((CKR_BUFFER_TOO_SMALL, template)),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn set_attribute_value(&self, session: CK_SESSION_HANDLE, object: CK_OBJECT_HANDLE, template: &Vec<CK_ATTRIBUTE>) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_SetAttributeValue)(session, object, template.as_slice().as_ptr(), template.len()) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn find_objects_init(&self, session: CK_SESSION_HANDLE, template: &Vec<CK_ATTRIBUTE>) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_FindObjectsInit)(session, template.as_slice().as_ptr(), template.len()) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn find_objects(&self, session: CK_SESSION_HANDLE, max_object_count: CK_ULONG) -> Result<Vec<CK_OBJECT_HANDLE>, Error> {
    self.initialized()?;
    let mut list: Vec<CK_OBJECT_HANDLE> = Vec::with_capacity(max_object_count);
    let mut count: CK_ULONG = 0;
    match (self.C_FindObjects)(session, list.as_ptr(), max_object_count, &mut count) {
      CKR_OK => {
        unsafe {
          list.set_len(count);
        }
        Ok(list)
      }
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn find_objects_final(&self, session: CK_SESSION_HANDLE) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_FindObjectsFinal)(session) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn encrypt_init(&self, session: CK_SESSION_HANDLE, mechanism: &CK_MECHANISM, key: CK_OBJECT_HANDLE) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_EncryptInit)(session, mechanism, key) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn encrypt(&self, session: CK_SESSION_HANDLE, data: &Vec<CK_BYTE>) -> Result<Vec<CK_BYTE>, Error> {
    self.initialized()?;
    let mut encryptedDataLen: CK_ULONG = 0;
    match (self.C_Encrypt)(session, data.as_slice().as_ptr(), data.len(), ptr::null(), &mut encryptedDataLen) {
      CKR_OK => {
        let mut encryptedData: Vec<CK_BYTE> = Vec::with_capacity(encryptedDataLen);
        match (self.C_Encrypt)(session, data.as_slice().as_ptr(), data.len(), encryptedData.as_slice().as_ptr(), &encryptedDataLen) {
          CKR_OK => {
            unsafe {
              encryptedData.set_len(encryptedDataLen);
            }
            Ok(encryptedData)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn encrypt_update(&self, session: CK_SESSION_HANDLE, part: &Vec<CK_BYTE>) -> Result<Vec<CK_BYTE>, Error> {
    self.initialized()?;
    let mut encryptedPartLen: CK_ULONG = 0;
    match (self.C_EncryptUpdate)(session, part.as_slice().as_ptr(), part.len(), ptr::null(), &mut encryptedPartLen) {
      CKR_OK => {
        let mut encryptedPart: Vec<CK_BYTE> = Vec::with_capacity(encryptedPartLen);
        match (self.C_EncryptUpdate)(session, part.as_slice().as_ptr(), part.len(), encryptedPart.as_slice().as_ptr(), &encryptedPartLen) {
          CKR_OK => {
            unsafe {
              encryptedPart.set_len(encryptedPartLen);
            }
            Ok(encryptedPart)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn encrypt_final(&self, session: CK_SESSION_HANDLE) -> Result<Option<Vec<CK_BYTE>>, Error> {
    self.initialized()?;
    let mut lastEncryptedPartLen: CK_ULONG = 0;
    match (self.C_EncryptFinal)(session, ptr::null(), &mut lastEncryptedPartLen) {
      CKR_OK => {
        if lastEncryptedPartLen == 0 {
          Ok(None)
        } else {
          let mut lastEncryptedPart: Vec<CK_BYTE> = Vec::with_capacity(lastEncryptedPartLen);
          match (self.C_EncryptFinal)(session, lastEncryptedPart.as_slice().as_ptr(), &lastEncryptedPartLen) {
            CKR_OK => {
              unsafe {
                lastEncryptedPart.set_len(lastEncryptedPartLen);
              }
              Ok(Some(lastEncryptedPart))
            },
            err => Err(Error::Pkcs11(err)),
          }
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn decrypt_init(&self, session: CK_SESSION_HANDLE, mechanism: &CK_MECHANISM, key: CK_OBJECT_HANDLE) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_DecryptInit)(session, mechanism, key) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn decrypt(&self, session: CK_SESSION_HANDLE, encryptedData: &Vec<CK_BYTE>) -> Result<Vec<CK_BYTE>, Error> {
    self.initialized()?;
    let mut dataLen: CK_ULONG = 0;
    match (self.C_Decrypt)(session, encryptedData.as_slice().as_ptr(), encryptedData.len(), ptr::null(), &mut dataLen) {
      CKR_OK => {
        let mut data: Vec<CK_BYTE> = Vec::with_capacity(dataLen);
        match (self.C_Decrypt)(session, encryptedData.as_slice().as_ptr(), encryptedData.len(), data.as_slice().as_ptr(), &dataLen) {
          CKR_OK => {
            unsafe {
              data.set_len(dataLen);
            }
            Ok(data)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn decrypt_update(&self, session: CK_SESSION_HANDLE, encryptedPart: &Vec<CK_BYTE>) -> Result<Vec<CK_BYTE>, Error> {
    self.initialized()?;
    let mut partLen: CK_ULONG = 0;
    match (self.C_DecryptUpdate)(session, encryptedPart.as_slice().as_ptr(), encryptedPart.len(), ptr::null(), &mut partLen) {
      CKR_OK => {
        let mut part: Vec<CK_BYTE> = Vec::with_capacity(partLen);
        match (self.C_Decrypt)(session, encryptedPart.as_slice().as_ptr(), encryptedPart.len(), part.as_slice().as_ptr(), &partLen) {
          CKR_OK => {
            unsafe {
              part.set_len(partLen);
            }
            Ok(part)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn decrypt_final(&self, session: CK_SESSION_HANDLE) -> Result<Option<Vec<CK_BYTE>>, Error> {
    let mut lastPartLen: CK_ULONG = 0;
    match (self.C_DecryptFinal)(session, ptr::null(), &mut lastPartLen) {
      CKR_OK => {
        if lastPartLen == 0 {
          Ok(None)
        } else {
          let mut lastPart: Vec<CK_BYTE> = Vec::with_capacity(lastPartLen);
          match (self.C_DecryptFinal)(session, lastPart.as_slice().as_ptr(), &lastPartLen) {
            CKR_OK => {
              unsafe {
                lastPart.set_len(lastPartLen);
              }
              Ok(Some(lastPart))
            },
            err => Err(Error::Pkcs11(err)),
          }
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn digest_init(&self, session: CK_SESSION_HANDLE, mechanism: &CK_MECHANISM) -> Result<(), Error> {
    match (self.C_DigestInit)(session, mechanism) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn digest(&self, session: CK_SESSION_HANDLE, data: &Vec<CK_BYTE>) -> Result<Vec<CK_BYTE>, Error> {
    let mut digestLen: CK_ULONG = 0;
    match (self.C_Digest)(session, data.as_slice().as_ptr(), data.len(), ptr::null(), &mut digestLen) {
      CKR_OK => {
        let mut digest: Vec<CK_BYTE> = Vec::with_capacity(digestLen);
        match (self.C_Digest)(session, data.as_slice().as_ptr(), data.len(), digest.as_slice().as_ptr(), &digestLen) {
          CKR_OK => {
            unsafe {
              digest.set_len(digestLen);
            }
            Ok(digest)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn digest_update(&self, session: CK_SESSION_HANDLE, part: &Vec<CK_BYTE>) -> Result<(), Error> {
    match (self.C_DigestUpdate)(session, part.as_slice().as_ptr(), part.len()) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn digest_key(&self, session: CK_SESSION_HANDLE, key: CK_OBJECT_HANDLE) -> Result<(), Error> {
    match (self.C_DigestKey)(session, key) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn digest_final(&self, session: CK_SESSION_HANDLE) -> Result<Vec<CK_BYTE>, Error> {
    let mut digestLen: CK_ULONG = 0;
    match (self.C_DigestFinal)(session, ptr::null(), &mut digestLen) {
      CKR_OK => {
        let mut digest: Vec<CK_BYTE> = Vec::with_capacity(digestLen);
        match (self.C_DigestFinal)(session, digest.as_slice().as_ptr(), &digestLen) {
          CKR_OK => {
            unsafe {
              digest.set_len(digestLen);
            }
            Ok(digest)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn sign_init(&self, session: CK_SESSION_HANDLE, mechanism: &CK_MECHANISM, key: CK_OBJECT_HANDLE) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_SignInit)(session, mechanism, key) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn sign(&self, session: CK_SESSION_HANDLE, data: &Vec<CK_BYTE>) -> Result<Vec<CK_BYTE>, Error> {
    self.initialized()?;
    let mut signatureLen: CK_ULONG = 0;
    match (self.C_Sign)(session, data.as_slice().as_ptr(), data.len(), ptr::null(), &mut signatureLen) {
      CKR_OK => {
        let mut signature: Vec<CK_BYTE> = Vec::with_capacity(signatureLen);
        match (self.C_Sign)(session, data.as_slice().as_ptr(), data.len(), ptr::null(), &signatureLen) {
          CKR_OK => {
            unsafe {
              signature.set_len(signatureLen); 
            }
            Ok(signature)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn sign_update(&self, session: CK_SESSION_HANDLE, part: &Vec<CK_BYTE>) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_SignUpdate)(session, part.as_slice().as_ptr(), part.len()) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn sign_final(&self, session: CK_SESSION_HANDLE) -> Result<Vec<CK_BYTE>, Error> {
    self.initialized()?;
    let mut signatureLen: CK_ULONG = 0;
    match (self.C_SignFinal)(session, ptr::null(), &mut signatureLen) {
      CKR_OK => {
        let mut signature: Vec<CK_BYTE> = Vec::with_capacity(signatureLen);
        match (self.C_SignFinal)(session, signature.as_slice().as_ptr(), &signatureLen) {
          CKR_OK => {
            unsafe {
              signature.set_len(signatureLen);
            }
            Ok(signature)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn sign_recover_init(&self, session: CK_SESSION_HANDLE, mechanism: &CK_MECHANISM, key: CK_OBJECT_HANDLE) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_SignRecoverInit)(session, mechanism, key) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err))
    }
  }

  pub fn sign_recover(&self, session: CK_SESSION_HANDLE, data: &Vec<CK_BYTE>) -> Result<Vec<CK_BYTE>, Error> {
    self.initialized()?;
    let mut signatureLen: CK_ULONG = 0;
    match (self.C_SignRecover)(session, data.as_slice().as_ptr(), data.len(), ptr::null(), &mut signatureLen) {
      CKR_OK => {
        let mut signature: Vec<CK_BYTE> = Vec::with_capacity(signatureLen);
        match (self.C_SignRecover)(session, data.as_slice().as_ptr(), data.len(), signature.as_slice().as_ptr(), &signatureLen) {
          CKR_OK => {
            unsafe {
              signature.set_len(signatureLen);
            }
            Ok(signature)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn verify_init(&self, session: CK_SESSION_HANDLE, mechanism: &CK_MECHANISM, key: CK_OBJECT_HANDLE) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_VerifyInit)(session, mechanism, key) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn verify(&self, session: CK_SESSION_HANDLE, data: &Vec<CK_BYTE>, signature: &Vec<CK_BYTE>) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_Verify)(session, data.as_slice().as_ptr(), data.len(), signature.as_slice().as_ptr(), signature.len()) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn verify_update(&self, session: CK_SESSION_HANDLE, part: &Vec<CK_BYTE>) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_VerifyUpdate)(session, part.as_slice().as_ptr(), part.len()) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn verify_final(&self, session: CK_SESSION_HANDLE, signature: &Vec<CK_BYTE>) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_VerifyFinal)(session, signature.as_slice().as_ptr(), signature.len()) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn verify_recover_init(&self, session: CK_SESSION_HANDLE, mechanism: &CK_MECHANISM, key: CK_OBJECT_HANDLE) -> Result<(), Error> {
    self.initialized()?;
    match (self.C_VerifyRecoverInit)(session, mechanism, key) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn verify_recover(&self, session: CK_SESSION_HANDLE, signature: &Vec<CK_BYTE>) -> Result<Vec<CK_BYTE>, Error> {
    self.initialized()?;
    let mut dataLen: CK_ULONG = 0;
    match (self.C_VerifyRecover)(session, signature.as_slice().as_ptr(), signature.len(), ptr::null(), &mut dataLen) {
      CKR_OK => {
        let mut data: Vec<CK_BYTE> = Vec::with_capacity(dataLen);
        match (self.C_VerifyRecover)(session, signature.as_slice().as_ptr(), signature.len(), data.as_slice().as_ptr(), &dataLen) {
          CKR_OK => {
            unsafe {
              data.set_len(dataLen);
            }
            Ok(data)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn digest_encrypt_update(&self, session: CK_SESSION_HANDLE, part: &Vec<CK_BYTE>) -> Result<Vec<CK_BYTE>, Error> {
    self.initialized()?;
    let mut encryptedPartLen: CK_ULONG = 0;
    match (self.C_DigestEncryptUpdate)(session, part.as_slice().as_ptr(), part.len(), ptr::null(), &mut encryptedPartLen) {
      CKR_OK => {
        let mut encryptedPart: Vec<CK_BYTE> = Vec::with_capacity(encryptedPartLen);
        match (self.C_DigestEncryptUpdate)(session, part.as_slice().as_ptr(), part.len(), encryptedPart.as_slice().as_ptr(), &encryptedPartLen) {
          CKR_OK => {
            unsafe {
              encryptedPart.set_len(encryptedPartLen);
            }
            Ok(encryptedPart)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn decrypt_digest_update(&self, session: CK_SESSION_HANDLE, encryptedPart: &Vec<CK_BYTE>) -> Result<Vec<CK_BYTE>, Error> {
    self.initialized()?;
    let mut partLen: CK_ULONG = 0;
    match (self.C_DecryptDigestUpdate)(session, encryptedPart.as_slice().as_ptr(), encryptedPart.len(), ptr::null(), &mut partLen) {
      CKR_OK => {
        let mut part: Vec<CK_BYTE> = Vec::with_capacity(partLen);
        match (self.C_DecryptDigestUpdate)(session, encryptedPart.as_slice().as_ptr(), encryptedPart.len(), part.as_slice().as_ptr(), &partLen) {
          CKR_OK => {
            unsafe {
              part.set_len(partLen);
            }
            Ok(part)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn sign_encrypt_update(&self, session: CK_SESSION_HANDLE, part: &Vec<CK_BYTE>) -> Result<Vec<CK_BYTE>, Error> {
    self.initialized()?;
    let mut encryptedPartLen: CK_ULONG = 0;
    match (self.C_SignEncryptUpdate)(session, part.as_slice().as_ptr(), part.len(), ptr::null(), &mut encryptedPartLen) {
      CKR_OK => {
        let mut encryptedPart: Vec<CK_BYTE> = Vec::with_capacity(encryptedPartLen);
        match (self.C_SignEncryptUpdate)(session, part.as_slice().as_ptr(), part.len(), encryptedPart.as_slice().as_ptr(), &encryptedPartLen) {
          CKR_OK => {
            unsafe {
              encryptedPart.set_len(encryptedPartLen);
            }
            Ok(encryptedPart)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn decrypt_verify_update(&self, session: CK_SESSION_HANDLE, encryptedPart: Vec<CK_BYTE>) -> Result<Vec<CK_BYTE>, Error> {
    self.initialized()?;
    let mut partLen: CK_ULONG = 0;
    match (self.C_DecryptVerifyUpdate)(session, encryptedPart.as_slice().as_ptr(), encryptedPart.len(), ptr::null(), &mut partLen) {
      CKR_OK => {
        let mut part: Vec<CK_BYTE> = Vec::with_capacity(partLen);
        match (self.C_DecryptVerifyUpdate)(session, encryptedPart.as_slice().as_ptr(), encryptedPart.len(), part.as_slice().as_ptr(), &partLen) {
          CKR_OK => {
            unsafe {
              part.set_len(partLen);
            }
            Ok(part)
          },
          err => Err(Error::Pkcs11(err)),
        }
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn generate_key(&self, session: CK_SESSION_HANDLE, mechanism: &CK_MECHANISM, template: &Vec<CK_ATTRIBUTE>) -> Result<CK_OBJECT_HANDLE, Error> {
    self.initialized()?;
    let mut object: CK_OBJECT_HANDLE = CK_INVALID_HANDLE;
    match (self.C_GenerateKey)(session, mechanism, template.as_slice().as_ptr(), template.len(), &mut object) {
      CKR_OK => Ok(object),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn generate_key_pair(
    &self,
    session: CK_SESSION_HANDLE,
    mechanism: &CK_MECHANISM,
    publicKeyTemplate: &Vec<CK_ATTRIBUTE>,
    privateKeyTemplate: &Vec<CK_ATTRIBUTE>,
  ) -> Result<(CK_OBJECT_HANDLE, CK_OBJECT_HANDLE), Error> {
    let mut pubOh: CK_OBJECT_HANDLE = CK_INVALID_HANDLE;
    let mut privOh: CK_OBJECT_HANDLE = CK_INVALID_HANDLE;
    match (self.C_GenerateKeyPair)(
      session,
      mechanism,
      publicKeyTemplate.as_slice().as_ptr(),
      publicKeyTemplate.len(),
      privateKeyTemplate.as_slice().as_ptr(),
      privateKeyTemplate.len(),
      &mut pubOh,
      &mut privOh,
    ) {
      CKR_OK => Ok((pubOh, privOh)),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn wrap_key(&self, session: CK_SESSION_HANDLE, mechanism: &CK_MECHANISM, wrappingKey: CK_OBJECT_HANDLE, key: CK_OBJECT_HANDLE) -> Result<Vec<CK_BYTE>, Error> {
    let mut length: CK_ULONG = 0;
    match (self.C_WrapKey)(session, mechanism, wrappingKey, key, ptr::null(), &mut length) {
      CKR_OK => if length > 0 {
        let mut out: Vec<CK_BYTE> = Vec::with_capacity(length);
        match (self.C_WrapKey)(session, mechanism, wrappingKey, key, out.as_slice().as_ptr(), &length) {
          CKR_OK => {
            unsafe {
              out.set_len(length);
            }
            Ok((out))
          }
          err => Err(Error::Pkcs11(err)),
        }
      } else {
        Ok((vec![]))
      },
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn unwrap_key(
    &self,
    session: CK_SESSION_HANDLE,
    mechanism: &CK_MECHANISM,
    unwrappingKey: CK_OBJECT_HANDLE,
    wrappedKey: &Vec<CK_BYTE>,
    template: &Vec<CK_ATTRIBUTE>,
  ) -> Result<CK_OBJECT_HANDLE, Error> {
    let mut oh: CK_OBJECT_HANDLE = CK_INVALID_HANDLE;
    match (self.C_UnwrapKey)(session, mechanism, unwrappingKey, wrappedKey.as_slice().as_ptr(), wrappedKey.len(), template.as_slice().as_ptr(), template.len(), &mut oh) {
      CKR_OK => Ok(oh),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn derive_key(&self, session: CK_SESSION_HANDLE, mechanism: &CK_MECHANISM, baseKey: CK_OBJECT_HANDLE, template: &Vec<CK_ATTRIBUTE>) -> Result<CK_OBJECT_HANDLE, Error> {
    let mut oh: CK_OBJECT_HANDLE = CK_INVALID_HANDLE;
    match (self.C_DeriveKey)(session, mechanism, baseKey, template.as_slice().as_ptr(), template.len(), &mut oh) {
      CKR_OK => Ok(oh),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn seed_random(&self, session: CK_SESSION_HANDLE, seed: &Vec<CK_BYTE>) -> Result<(), Error> {
    match (self.C_SeedRandom)(session, seed.as_slice().as_ptr(), seed.len()) {
      CKR_OK => Ok(()),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn generate_random(&self, session: CK_SESSION_HANDLE, randomLength: CK_ULONG) -> Result<Vec<CK_BYTE>, Error> {
    let mut data: Vec<CK_BYTE> = Vec::with_capacity(randomLength);
    match (self.C_GenerateRandom)(session, data.as_slice().as_ptr(), randomLength) {
      CKR_OK => {
        unsafe {
          data.set_len(randomLength);
        }
        Ok(data)
      }
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn get_function_status(&self, session: CK_SESSION_HANDLE) -> Result<CK_RV, Error> {
    match (self.C_GetFunctionStatus)(session) {
      CKR_OK => Ok(CKR_OK),
      CKR_FUNCTION_NOT_PARALLEL => Ok(CKR_FUNCTION_NOT_PARALLEL),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn cancel_function(&self, session: CK_SESSION_HANDLE) -> Result<CK_RV, Error> {
    match (self.C_CancelFunction)(session) {
      CKR_OK => Ok(CKR_OK),
      CKR_FUNCTION_NOT_PARALLEL => Ok(CKR_FUNCTION_NOT_PARALLEL),
      err => Err(Error::Pkcs11(err)),
    }
  }

  pub fn wait_for_slot_event(&self, flags: CK_FLAGS) -> Result<CK_SLOT_ID, Error> {
    let mut slotID: CK_SLOT_ID = 0;
    match (self.C_WaitForSlotEvent)(flags, &mut slotID, ptr::null()) {
      CKR_OK => Ok(slotID),
      err => Err(Error::Pkcs11(err)),
    }
  }
}

impl Drop for Ctx {
  fn drop(&mut self) {
    if self.is_initialized() {
      if let Err(err) = self.finalize() {
        println!("ERROR: {}", err);
      }
    }
  }
}
