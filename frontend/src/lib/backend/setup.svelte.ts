import { post, RequestError, ResponseType } from 'positron-components/backend';
import { fetchKey, getEncrypt } from './auth.svelte';

export interface SetupPayload {
  admin_username: string;
  admin_password: string;
  admin_email: string;
}

export interface SetupResponse {
  user: string;
}

export const performSetup = async (payload: SetupPayload) => {
  let encrypt = getEncrypt();
  if (!encrypt) {
    return RequestError.Other;
  }

  let encrypted_password = encrypt.encrypt(payload.admin_password);
  payload.admin_password = encrypted_password || '';

  let res = await post<SetupResponse>('/api/setup', {
    res_type: ResponseType.Json,
    body: payload
  });

  if (res === RequestError.Unauthorized) {
    fetchKey();
  }

  return res;
};
