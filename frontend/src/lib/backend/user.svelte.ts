import {
  post,
  put,
  RequestError,
  ResponseType
} from 'positron-components/backend';
import { fetchKey, getEncrypt } from './auth.svelte';

export interface UpdatePassword {
  old_password: string;
  new_password: string;
}

export const updatePassword = async (payload: UpdatePassword) => {
  let encrypt = getEncrypt();
  if (!encrypt) {
    return RequestError.Other;
  }

  let encrypted_password = encrypt.encrypt(payload.old_password);
  payload.old_password = encrypted_password || '';
  encrypted_password = encrypt.encrypt(payload.new_password);
  payload.new_password = encrypted_password || '';

  let res = await post('/api/user/account/password', {
    body: payload
  });

  if (res === RequestError.Unauthorized) {
    fetchKey();
  }

  return res;
};

export interface CreateUserRequest {
  name: string;
  email: string;
  password?: string;
}

export const createUser = async (data: CreateUserRequest) => {
  let encrypt = getEncrypt();
  if (!encrypt) {
    return RequestError.Other;
  }

  let encrypted_password = encrypt.encrypt(data.password || '');
  data.password = encrypted_password || '';

  let res = await post<{ uuid: string }>('/api/user/management', {
    body: data,
    res_type: ResponseType.Json
  });

  if (res === RequestError.Unauthorized) {
    fetchKey();
  }

  return res;
};

export interface ResetUserPasswordRequest {
  uuid: string;
  new_password: string;
}

export const resetUserPassword = async (data: ResetUserPasswordRequest) => {
  let encrypt = getEncrypt();
  if (!encrypt) {
    return RequestError.Other;
  }

  let encrypted_password = encrypt.encrypt(data.new_password);
  data.new_password = encrypted_password || '';

  let res = await put('/api/user/management/password', {
    body: data
  });

  if (res === RequestError.Unauthorized) {
    fetchKey();
  }

  return res;
};
