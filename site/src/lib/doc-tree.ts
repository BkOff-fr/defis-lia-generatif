interface DocEntry {
  id: string;
  data: { title?: string; sourcePath: string; order?: number; category?: string };
  body?: string;
}

export interface TreeNode {
  type: 'folder' | 'leaf';
  name: string;
  href?: string;
  title: string;
  children?: TreeNode[];
}

export function firstH1(body: string | undefined): string | null {
  if (!body) return null;
  const m = body.match(/^#\s+(.+)$/m);
  return m ? m[1].trim() : null;
}

export function getEntryTitle(entry: DocEntry): string {
  if (entry.data.title) return entry.data.title;
  const h1 = firstH1(entry.body);
  if (h1) return h1;
  const last = entry.id.split('/').pop() || entry.id;
  return last
    .replace(/[-_]/g, ' ')
    .replace(/\.md$/i, '')
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

export function buildTree(entries: DocEntry[], baseHref: string): TreeNode[] {
  const folders: Record<string, TreeNode> = {};
  const root: TreeNode[] = [];

  const sorted = [...entries].sort((a, b) => {
    const ao = a.data.order ?? 999;
    const bo = b.data.order ?? 999;
    if (ao !== bo) return ao - bo;
    return getEntryTitle(a).localeCompare(getEntryTitle(b));
  });

  for (const entry of sorted) {
    const segments = entry.id.split('/');
    if (segments.length === 1) {
      root.push({
        type: 'leaf',
        name: segments[0],
        href: `${baseHref}/${entry.id}/`,
        title: getEntryTitle(entry),
      });
    } else {
      const folderName = segments[0];
      if (!folders[folderName]) {
        folders[folderName] = {
          type: 'folder',
          name: folderName,
          title: folderName.replace(/[-_]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase()),
          children: [],
        };
        root.push(folders[folderName]);
      }
      folders[folderName].children!.push({
        type: 'leaf',
        name: segments.slice(1).join('/'),
        href: `${baseHref}/${entry.id}/`,
        title: getEntryTitle(entry),
      });
    }
  }
  return root;
}
