import { z } from 'zod';

export const cacheSchema = z.object({
  name: z.string().min(1, { message: 'Name is required' }),
  priority: z
    .number()
    .int()
    .min(0, { message: 'Priority must be a non-negative integer' }),
  allow_force_push: z.boolean().default(false)
});
