import { useRef, type ChangeEvent } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Link, useParams } from 'react-router-dom';
import {
  getGatheringByInviteCode,
  listPhotos,
  uploadPhoto,
} from '../api/gatherings';
import { listMenuItems } from '../api/menuItems';
import DishCard from '../components/DishCard';
import PageCard from '../components/PageCard';
import StatusPill from '../components/StatusPill';
import { mockPhotos } from '../data/mockGathering';

export default function ReviewPage() {
  const { inviteCode } = useParams();
  const queryClient = useQueryClient();
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const gatheringQuery = useQuery({
    queryKey: ['gathering', inviteCode],
    queryFn: () => getGatheringByInviteCode(inviteCode ?? ''),
    enabled: Boolean(inviteCode),
    retry: false,
  });
  const gathering = gatheringQuery.data?.gathering;
  const menuItemsQuery = useQuery({
    queryKey: ['menu-items', gathering?.id],
    queryFn: () => listMenuItems(gathering?.id ?? ''),
    enabled: Boolean(gathering?.id && gathering?.is_locked),
    retry: false,
  });
  const finalMenuItems =
    menuItemsQuery.data?.menu_items.filter((item) => item.status !== 'cancelled') ??
    [];
  const photosQuery = useQuery({
    queryKey: ['photos', gathering?.id],
    queryFn: () => listPhotos(gathering?.id ?? ''),
    enabled: Boolean(gathering?.id && gathering?.is_locked),
    retry: false,
  });
  const photoUploadMutation = useMutation({
    mutationFn: (file: File) => uploadPhoto(gathering?.id ?? '', file),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['photos', gathering?.id] });
      await queryClient.invalidateQueries({ queryKey: ['activity-logs', gathering?.id] });
    },
  });
  const uploadedPhotos = photosQuery.data?.photos ?? [];

  function handlePhotoSelected(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file || !gathering?.id) {
      return;
    }

    photoUploadMutation.mutate(file);
    event.target.value = '';
  }

  if (gathering && !gathering.is_locked) {
    return (
      <PageCard
        eyebrow="Gathering archive"
        title="Review is not ready yet"
        description="The final menu becomes available after this gathering is locked."
      >
        <div className="action-row">
          <Link className="button-link secondary" to={`/menu/${inviteCode}`}>
            Back to menu
          </Link>
          <Link className="button-link secondary" to={`/host/${inviteCode}`}>
            On Track
          </Link>
        </div>
      </PageCard>
    );
  }

  return (
    <div className="review-layout">
      <PageCard
        eyebrow="Gathering archive"
        title={gathering ? `${gathering.title} review` : 'Review'}
        description="After the menu locks, this page keeps the final menu and photo memories together."
      >
        <div className="action-row">
          <StatusPill tone="neutral">Read-only menu</StatusPill>
          <Link className="button-link secondary" to={`/menu/${inviteCode}`}>
            Back to menu
          </Link>
        </div>
      </PageCard>

      <section className="section-block">
        <div className="panel-header">
          <div>
            <p className="card-kicker">Final menu</p>
            <h2>What made it to the table</h2>
          </div>
        </div>
        <div className="dish-list final-menu-list">
          {finalMenuItems.map((item) => (
            <DishCard item={item} key={item.id} readOnly />
          ))}
          {finalMenuItems.length === 0 ? (
            <p className="empty-panel-note">No final menu items yet.</p>
          ) : null}
        </div>
      </section>

      <section className="section-block">
        <div className="panel-header">
          <div>
            <p className="card-kicker">Photo wall</p>
            <h2>Little memories, neatly kept</h2>
          </div>
          <button type="button" onClick={() => fileInputRef.current?.click()}>
            {photoUploadMutation.isPending ? 'Uploading...' : 'Upload photos'}
          </button>
          <input
            ref={fileInputRef}
            accept="image/*"
            hidden
            type="file"
            onChange={handlePhotoSelected}
          />
        </div>
        {photoUploadMutation.isError ? (
          <p className="error">Could not upload this photo.</p>
        ) : null}
        <div className="photo-grid">
          <article className="photo-card uploaded-photo-card">
            <img alt="Mock gathering memory" src="/resources/mock/mock-gathering-photo.svg" />
            <p>Mock gathering photo</p>
          </article>
          {uploadedPhotos.map((photo) => (
            <article className="photo-card uploaded-photo-card" key={photo.id}>
              <img alt={photo.caption ?? 'Uploaded gathering memory'} src={photo.file_url} />
              <p>{photo.caption ?? 'Uploaded memory'}</p>
            </article>
          ))}
          {mockPhotos.map((photo) => (
            <article className={`photo-card photo-${photo.color}`} key={photo.id}>
              <div className="photo-blob" />
              <p>{photo.title}</p>
            </article>
          ))}
          <article className="upload-card">
            <strong>Drop photos here</strong>
            <span>JPG, PNG, or HEIC later</span>
          </article>
        </div>
      </section>
    </div>
  );
}
