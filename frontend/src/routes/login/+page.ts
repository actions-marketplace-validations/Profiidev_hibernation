import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { authConfig, oidcUrl, SsoType } from '$lib/client';

export const load: PageLoad = async ({ fetch, url }) => {
  let error = url.searchParams.get('error') || null;
  if (error) {
    return { error };
  }
  let skip = url.searchParams.get('skip') === 'true';

  let { data: config } = await authConfig({ fetch });
  if (config?.sso_type !== SsoType.NONE) {
    let url = (
      (await oidcUrl({ fetch })) as { data: { url: string } } | undefined
    )?.data.url;
    if (url && config?.instant_redirect && !skip) {
      redirect(302, url);
    }
    return { oidc_url: url, config, skip };
  }
  return { config, skip };
};
