import type JSEncrypt from 'jsencrypt';
import { RequestError } from 'positron-components/backend';
import { browser } from '$app/environment';
import { key as getKey } from '$lib/client';

let encrypt: false | undefined | JSEncrypt = $state(browser && undefined);

export const getEncrypt = () => {
  return encrypt;
};

export const fetchKey = async () => {
  if (encrypt === false) {
    return RequestError.Other;
  }

  let { data: keyData } = await getKey();
  if (!keyData) {
    return;
  }

  const JSEncrypt = (await import('jsencrypt')).JSEncrypt;

  encrypt = new JSEncrypt({ default_key_size: '4096' });
  encrypt.setPublicKey(keyData.key);
};
fetchKey();
