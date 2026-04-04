import { get, post, RequestError } from 'positron-components/backend';
import { fetchKey, getEncrypt } from './auth.svelte';

export interface ResetPassword {
  token: string;
  new_password: string;
}

export const sendResetPassword = async (data: ResetPassword) => {
  let encrypt = getEncrypt();
  if (!encrypt) {
    return RequestError.Other;
  }

  let encrypted_password = encrypt.encrypt(data.new_password);
  data.new_password = encrypted_password || '';

  let res = await post(`/api/mail/reset/confirm`, {
    body: data
  });
  if (res === RequestError.Unauthorized) {
    fetchKey();
  }

  return res;
};

export const size_to_gib = (size: number) => {
  return size / 1024 / 1024 / 1024;
};

export const sendCliCode = async (code: string, user: string) => {
  return await get('http://localhost:16401?code=' + code);
};
