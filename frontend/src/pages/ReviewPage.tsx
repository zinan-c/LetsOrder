import { useRef, useState, type ChangeEvent } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Link, useParams } from 'react-router-dom';
import {
  deletePhoto,
  getGatheringByInviteCode,
  listPhotos,
  updatePhotoCaption,
  uploadPhoto,
} from '../api/gatherings';
import { listMenuItems, listMenuRatings, rateMenuItem } from '../api/menuItems';
import DishCard from '../components/DishCard';
import PageCard from '../components/PageCard';
import StatusPill from '../components/StatusPill';
import { getCurrentUser } from '../utils/user';

export default function ReviewPage() {
  const { inviteCode } = useParams();
  const queryClient = useQueryClient();
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const [photoTitle, setPhotoTitle] = useState('');
  const [editingPhotoId, setEditingPhotoId] = useState<string | null>(null);
  const [editingPhotoTitle, setEditingPhotoTitle] = useState('');
  const isAdmin = getCurrentUser()?.role === 'admin';
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
  const ratingsQuery = useQuery({
    queryKey: ['menu-ratings', gathering?.id],
    queryFn: () => listMenuRatings(gathering?.id ?? ''),
    enabled: Boolean(gathering?.id && gathering?.is_locked),
    retry: false,
  });
  const ratingsByMenuItem = new Map(
    (ratingsQuery.data?.ratings ?? []).map((rating) => [
      rating.menu_item_id,
      rating,
    ]),
  );
  const photosQuery = useQuery({
    queryKey: ['photos', gathering?.id],
    queryFn: () => listPhotos(gathering?.id ?? ''),
    enabled: Boolean(gathering?.id && gathering?.is_locked),
    retry: false,
  });
  const photoUploadMutation = useMutation({
    mutationFn: ({ file, caption }: { file: File; caption?: string }) =>
      uploadPhoto(gathering?.id ?? '', file, caption),
    onSuccess: async () => {
      setPhotoTitle('');
      await queryClient.invalidateQueries({ queryKey: ['photos', gathering?.id] });
      await queryClient.invalidateQueries({ queryKey: ['activity-logs', gathering?.id] });
    },
  });
  const photoCaptionMutation = useMutation({
    mutationFn: ({ photoId, caption }: { photoId: string; caption: string }) =>
      updatePhotoCaption(photoId, caption),
    onSuccess: async () => {
      setEditingPhotoId(null);
      setEditingPhotoTitle('');
      await queryClient.invalidateQueries({ queryKey: ['photos', gathering?.id] });
      await queryClient.invalidateQueries({ queryKey: ['activity-logs', gathering?.id] });
    },
  });
  const photoDeleteMutation = useMutation({
    mutationFn: deletePhoto,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['photos', gathering?.id] });
      await queryClient.invalidateQueries({ queryKey: ['activity-logs', gathering?.id] });
    },
  });
  const ratingMutation = useMutation({
    mutationFn: ({ menuItemId, rating }: { menuItemId: string; rating: number }) =>
      rateMenuItem(menuItemId, rating),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['menu-ratings', gathering?.id] });
      await queryClient.invalidateQueries({ queryKey: ['activity-logs', gathering?.id] });
    },
  });
  const uploadedPhotos = photosQuery.data?.photos ?? [];

  if (gatheringQuery.isLoading) {
    return <PageCard eyebrow="Gathering archive" title="Loading review" description="Loading the final menu and photo wall..." />;
  }

  if (gatheringQuery.isError || !gathering) {
    return <PageCard eyebrow="Gathering archive" title="Review unavailable" description="This gathering could not be loaded. Check the invitation or try again later." />;
  }

  function handlePhotoSelected(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file || !gathering?.id) {
      return;
    }

    photoUploadMutation.mutate({
      file,
      caption: photoTitle.trim() || undefined,
    });
    event.target.value = '';
  }

  function startEditingPhoto(photoId: string, caption?: string | null) {
    setEditingPhotoId(photoId);
    setEditingPhotoTitle(caption ?? 'Image');
  }

  function savePhotoTitle() {
    if (!editingPhotoId) {
      return;
    }

    photoCaptionMutation.mutate({
      photoId: editingPhotoId,
      caption: editingPhotoTitle,
    });
  }

  function confirmDeletePhoto(photoId: string) {
    if (window.confirm('Delete this photo?')) {
      photoDeleteMutation.mutate(photoId);
    }
  }

  if (!gathering.is_locked) {
    return (
      <PageCard
        eyebrow="Gathering archive"
        title="Review is not ready yet"
        description="The final menu becomes available after this gathering is locked."
      >
        <div className="action-row">
          <Link
            className="button-link secondary"
            to={`/menu/${inviteCode}?from=review`}
          >
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
          <Link
            className="button-link secondary"
            to={`/menu/${inviteCode}?from=review`}
          >
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
          {finalMenuItems.map((item) => {
            const rating = ratingsByMenuItem.get(item.id);
            const averageRating =
              typeof rating?.average_rating === 'number'
                ? rating.average_rating.toFixed(1)
                : 'No ratings yet';

            return (
              <article className="rated-dish-card" key={item.id}>
                <DishCard item={item} readOnly />
                <div className="rating-panel">
                  <div>
                    <p className="card-kicker">Rating</p>
                    <strong>{averageRating}</strong>
                    <span>
                      {rating?.rating_count
                        ? `${rating.rating_count} rating${rating.rating_count === 1 ? '' : 's'}`
                        : 'Be the first to rate'}
                    </span>
                  </div>
                  {isAdmin ? (
                    <span className="rating-readonly-note">Participants can rate dishes.</span>
                  ) : (
                    <div className="star-rating" aria-label={`Rate ${item.name}`}>
                      {[1, 2, 3, 4, 5].map((score) => (
                        <button
                          aria-label={`Rate ${item.name} ${score} star${score === 1 ? '' : 's'}`}
                          className={
                            score <= (rating?.my_rating ?? 0)
                              ? 'star-button active'
                              : 'star-button'
                          }
                          disabled={ratingMutation.isPending}
                          key={score}
                          type="button"
                          onClick={() =>
                            ratingMutation.mutate({
                              menuItemId: item.id,
                              rating: score,
                            })
                          }
                        >
                          ★
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              </article>
            );
          })}
          {finalMenuItems.length === 0 ? (
            <p className="empty-panel-note">No final menu items yet.</p>
          ) : null}
        </div>
        {ratingMutation.isError ? (
          <p className="error">Could not save this rating.</p>
        ) : null}
      </section>

      <section className="section-block">
        <div className="panel-header">
          <div>
            <p className="card-kicker">Photo wall</p>
            <h2>Little memories, neatly kept</h2>
          </div>
          <div className="photo-upload-controls">
            <label>
              Photo title
              <input
                value={photoTitle}
                placeholder="Grandma's soup moment"
                onChange={(event) => setPhotoTitle(event.target.value)}
              />
            </label>
            <button type="button" onClick={() => fileInputRef.current?.click()}>
              {photoUploadMutation.isPending ? 'Uploading...' : 'Upload photos'}
            </button>
          </div>
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
        {photoCaptionMutation.isError ? (
          <p className="error">Could not update this photo title.</p>
        ) : null}
        {photoDeleteMutation.isError ? (
          <p className="error">Could not delete this photo.</p>
        ) : null}
        {uploadedPhotos.length === 0 ? (
          <p className="empty-panel-note">
            Are you ready to take notes of the photos?
          </p>
        ) : null}
        {uploadedPhotos.length > 0 ? (
          <div className="photo-grid">
            {uploadedPhotos.map((photo) => (
              <article className="photo-card uploaded-photo-card" key={photo.id}>
                <img
                  alt={photo.caption ?? 'Uploaded gathering memory'}
                  src={photo.file_url}
                />
                {editingPhotoId === photo.id ? (
                  <div className="photo-admin-editor">
                    <input
                      value={editingPhotoTitle}
                      onChange={(event) => setEditingPhotoTitle(event.target.value)}
                    />
                    <div className="action-row">
                      <button
                        disabled={photoCaptionMutation.isPending}
                        type="button"
                        onClick={savePhotoTitle}
                      >
                        Save
                      </button>
                      <button
                        className="ghost-button"
                        disabled={photoCaptionMutation.isPending}
                        type="button"
                        onClick={() => setEditingPhotoId(null)}
                      >
                        Cancel
                      </button>
                    </div>
                  </div>
                ) : (
                  <p>{photo.caption ?? 'Image'}</p>
                )}
                {isAdmin && editingPhotoId !== photo.id ? (
                  <div className="photo-admin-actions">
                    <button
                      className="ghost-button"
                      type="button"
                      onClick={() => startEditingPhoto(photo.id, photo.caption)}
                    >
                      Edit title
                    </button>
                    <button
                      className="ghost-button danger-button"
                      disabled={photoDeleteMutation.isPending}
                      type="button"
                      onClick={() => confirmDeletePhoto(photo.id)}
                    >
                      Delete
                    </button>
                  </div>
                ) : null}
              </article>
            ))}
          </div>
        ) : null}
      </section>
    </div>
  );
}
