import { type FC, useState } from 'react';
import { LuTags } from 'react-icons/lu';
import { IconButton } from '../shared';

export type TagsEditorProps = {
  tags: string[];
  onUpdate: (tags: string[]) => void;
  iconClassName?: string;
};

export const TagsEditor: FC<TagsEditorProps> = ({
  tags,
  onUpdate,
  iconClassName = 'h-3.5 w-3.5',
}) => {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState('');
  if (!editing) {
    return (
      <IconButton
        label='Edit cell tags'
        icon={<LuTags aria-hidden='true' className={iconClassName} />}
        onClick={() => {
          setDraft(tags.join(', '));
          setEditing(true);
        }}
      />
    );
  }
  return (
    <input
      autoFocus
      aria-label='Cell tags'
      placeholder='tag1, tag2'
      value={draft}
      className='w-28 rounded border border-neutral-700 bg-neutral-950 px-1.5 py-0.5 text-xs text-neutral-200 outline-none'
      onClick={(event) => event.stopPropagation()}
      onChange={(event) => setDraft(event.target.value)}
      onBlur={() => {
        onUpdate(
          draft
            .split(',')
            .map((tag) => tag.trim())
            .filter(Boolean),
        );
        setEditing(false);
      }}
      onKeyDown={(event) => {
        if (event.key === 'Enter') event.currentTarget.blur();
        if (event.key === 'Escape') setEditing(false);
      }}
    />
  );
};

TagsEditor.displayName = 'TagsEditor';
