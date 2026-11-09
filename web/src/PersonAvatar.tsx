import { useEffect, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { avatarBlob } from './api';
import type { Person, AvatarMeta } from './types';
export default function PersonAvatar({
  person,
  meta,
  small = false,
}: {
  person: Person;
  meta?: AvatarMeta;
  small?: boolean;
}) {
  return (
    <span className={small ? 'mini-avatar person-avatar' : 'avatar person-avatar'}>
      {meta?.has_photo ? <Photo person={person} meta={meta} /> : person.name[0]}
    </span>
  );
}
function Photo({ person, meta }: { person: Person; meta: AvatarMeta }) {
  const [url, setUrl] = useState('');
  const [broken, setBroken] = useState(false);
  const photo = useQuery({
    queryKey: ['avatar', person.id, meta?.revision],
    queryFn: ({ signal }) => avatarBlob(person.id, signal),
    enabled: Boolean(meta?.has_photo),
    retry: false,
    staleTime: Infinity,
  });
  useEffect(() => {
    setBroken(false);
    if (!meta?.has_photo || !photo.data) {
      setUrl('');
      return;
    }
    const object = URL.createObjectURL(photo.data);
    setUrl(object);
    return () => URL.revokeObjectURL(object);
  }, [meta?.has_photo, meta?.revision, photo.data]);
  return (
    <>
      {url && !broken ? (
        <img src={url} alt={person.name} onError={() => setBroken(true)} />
      ) : (
        person.name[0]
      )}
    </>
  );
}
