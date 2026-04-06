import type { PageLoad } from './$types';
import { api } from '$lib/api';

export const load: PageLoad = async ({ params }) => {
  const campaign = await api.campaigns.get(params.id);
  return { campaign };
};
