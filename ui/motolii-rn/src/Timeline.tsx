import React, {useState} from 'react';
import {Pressable, Text, View} from 'react-native';

import MotoliiTimelineView from './specs/MotoliiTimelineViewNativeComponent';
import {
  applyNativeHostTerminal,
  dispatchHostIntent,
  formatPlayhead,
  playheadFraction,
  type HostRationalTime,
} from './host';
import {styles} from './productStyles';

const TRANSFORM_KEY_ACTIONS = [
  {
    testID: 'timeline-add-position-key',
    label: 'Position',
    kind: 'add_position_key',
  },
  {
    testID: 'timeline-add-scale-key',
    label: 'Scale',
    kind: 'add_param_key',
    property: 'scale',
  },
  {
    testID: 'timeline-add-rotation-key',
    label: 'Rotation',
    kind: 'add_param_key',
    property: 'rotation',
  },
  {
    testID: 'timeline-add-opacity-key',
    label: 'Opacity',
    kind: 'add_param_key',
    property: 'opacity',
  },
] as const;

export function TimelineKeyTools({
  primaryLayerId,
  currentTime,
}: {
  primaryLayerId: string | null;
  currentTime: HostRationalTime | null;
}) {
  const canAdd = primaryLayerId != null && currentTime != null;
  return (
    <View style={styles.timelineKeyTools} testID="timeline-key-tools">
      <View style={styles.timelineKeyActions}>
        <Text style={styles.timelineKeyActionTitle}>KEY TOOLS</Text>
        {TRANSFORM_KEY_ACTIONS.map(action => (
          <Pressable
            key={action.testID}
            accessibilityState={{disabled: !canAdd}}
            accessibilityRole="button"
            accessibilityLabel={`Add ${action.label} Key`}
            disabled={!canAdd}
            onPress={() => {
              if (!primaryLayerId || !currentTime) {
                return;
              }
              // Positionは専用intent。他は add_param_key + property。
              dispatchHostIntent(
                action.kind,
                'property' in action
                  ? {target: primaryLayerId, time: currentTime, property: action.property}
                  : {target: primaryLayerId, time: currentTime},
              );
            }}
            style={[
              styles.timelineKeyActionButton,
              !canAdd && styles.timelineKeyActionButtonDisabled,
            ]}
            testID={action.testID}>
            <Text style={[styles.timelineKeyGlyph, !canAdd && styles.timelineKeyGlyphDisabled]}>
              {action.label}
            </Text>
          </Pressable>
        ))}
        <Pressable
          accessibilityState={{disabled: !canAdd}}
          accessibilityRole="button"
          accessibilityLabel="Split clip"
          disabled={!canAdd}
          onPress={() => {
            if (!primaryLayerId || !currentTime) {
              return;
            }
            dispatchHostIntent('split', {
              target: primaryLayerId,
              time: currentTime,
            });
          }}
          style={[
            styles.timelineKeyActionButton,
            !canAdd && styles.timelineKeyActionButtonDisabled,
          ]}
          testID="timeline-split-clip">
          <Text style={[styles.timelineKeyGlyph, !canAdd && styles.timelineKeyGlyphDisabled]}>
            Split
          </Text>
        </Pressable>
        {!canAdd ? (
          <Text style={styles.timelineKeyHint} testID="timeline-key-tools-hint">
            {primaryLayerId == null ? 'Select a layer to edit keys' : 'Timeline time is unavailable'}
          </Text>
        ) : null}
      </View>
    </View>
  );
}

function formatTimelineFraction(
  fraction: number,
  duration: HostRationalTime | null,
): string {
  if (!duration || duration.den === 0) {
    return formatPlayhead(null);
  }
  const durationSeconds = duration.num / duration.den;
  if (!Number.isFinite(durationSeconds) || durationSeconds < 0) {
    return formatPlayhead(null);
  }
  const seconds = Math.max(0, Math.min(1, fraction)) * durationSeconds;
  return formatPlayhead({num: Math.round(seconds * 1000), den: 1000});
}

export function NativeTimeline({
  currentTime,
  duration,
  primaryLayerId,
}: {
  currentTime: HostRationalTime | null;
  duration: HostRationalTime | null;
  primaryLayerId: string | null;
}) {
  const playhead = playheadFraction(currentTime, duration);
  const [hitLabel, setHitLabel] = useState<string | null>(null);

  return (
    <View style={styles.nativeTimelineBody}>
      <View style={styles.nativeTimelineControls}>
        <Text style={styles.nativeTimelineTitle}>ARRANGEMENT</Text>
        <Text style={styles.nativeTimelineFeedback} testID="native-timeline-feedback">
          {hitLabel ?? formatPlayhead(currentTime)}
        </Text>
      </View>
      <View style={styles.nativeTimelineContent}>
        <MotoliiTimelineView
          accessible
          accessibilityLabel="Timeline arrangement"
          onTimelineFeedback={event => {
            const {objectIndex, time} = event.nativeEvent;
            const timecode = formatTimelineFraction(time, duration);
            setHitLabel(
              objectIndex >= 0
                ? `Clip ${objectIndex + 1} · ${timecode}`
                : timecode,
            );
          }}
          onHostTerminal={event => {
            const {accepted, message} = event.nativeEvent;
            applyNativeHostTerminal(accepted, message);
          }}
          playhead={playhead}
          selectedObjectIndex={-1}
          style={styles.nativeTimelineSurface}
          testID="rust-wgpu-timeline"
        />
        <TimelineKeyTools currentTime={currentTime} primaryLayerId={primaryLayerId} />
      </View>
    </View>
  );
}

export function Timeline({
  height,
  currentTime,
  duration,
  primaryLayerId,
}: {
  height: number;
  currentTime: HostRationalTime | null;
  duration: HostRationalTime | null;
  primaryLayerId: string | null;
}) {
  return (
    <View style={[styles.timeline, {height}]} testID="timeline">
      <View style={styles.timelineHeader}>
        <Text style={styles.panelTitle}>Timeline</Text>
        <Text style={styles.panelDetail}>{formatPlayhead(currentTime)}</Text>
      </View>
      <NativeTimeline
        currentTime={currentTime}
        duration={duration}
        primaryLayerId={primaryLayerId}
      />
    </View>
  );
}
