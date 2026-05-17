import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const docsSchema = z.object({
  sourcePath: z.string(),
  title: z.string().optional(),
  description: z.string().optional(),
  order: z.number().optional(),
  category: z.string().optional(),
});

export const collections = {
  docs: defineCollection({
    loader: glob({ pattern: '**/*.md', base: './src/content/docs' }),
    schema: docsSchema,
  }),
  adrs: defineCollection({
    loader: glob({ pattern: '**/*.md', base: './src/content/adrs' }),
    schema: docsSchema,
  }),
};
