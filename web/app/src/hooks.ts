import { useQuery } from '@tanstack/react-query';
import { getSpaceByKey } from './api';

export const useSpace = (key: string | undefined) => useQuery({ queryKey: ['space', key], queryFn: () => getSpaceByKey(key!), enabled: !!key });
