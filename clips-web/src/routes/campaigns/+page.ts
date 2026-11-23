import type { PageLoad } from './$types';
import { api } from '#lib/api.ts';

export const load: PageLoad = async () => {
  try {
    const campaigns = await api.campaigns.list();
    return { campaigns };
  } catch {
    return { campaigns: [] };
  }
};
