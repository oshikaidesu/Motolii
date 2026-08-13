import React from 'react';
import {Pressable, Text, View} from 'react-native';

import MotoliiGpuView from './specs/MotoliiGpuViewNativeComponent';
import {
  applyNativeHostTerminal,
  formatPlayhead,
  nativeHostFitStageView,
  nativeHostStageOneToOne,
  type HostRationalTime,
  type StageTransform,
} from './host';
import {styles} from './productStyles';

export function Stage({draggedItemId, showGpu, onDrop, onToggleGpu, transform, currentTime, playing, primaryLayerId, onTogglePlayback, onSkipStart, onSkipEnd, onMarkIn, onMarkOut}: {draggedItemId: string; showGpu: boolean; onDrop: (x: number, y: number, canonicalX: number, canonicalY: number) => void; onToggleGpu: () => void; transform: StageTransform; currentTime: HostRationalTime | null; playing: boolean; primaryLayerId: string | null; onTogglePlayback: () => void; onSkipStart: () => void; onSkipEnd: () => void; onMarkIn: () => void; onMarkOut: () => void}) {
  return (
    <View style={styles.stage} testID="stage-surface">
      <View style={styles.stageTools}>
        <Pressable accessibilityLabel="Fit Stage" onPress={nativeHostFitStageView}>
          <Text style={styles.stageToolText}>Fit</Text>
        </Pressable>
        <Pressable accessibilityLabel="Stage 100 percent" onPress={nativeHostStageOneToOne}>
          <Text style={styles.stageToolText}>100%</Text>
        </Pressable>
        <Pressable onPress={onToggleGpu} style={styles.stageGpuButton}>
          <Text style={styles.stageToolText}>{showGpu ? 'GPU ON' : 'GPU OFF'}</Text>
        </Pressable>
        <Text style={styles.stageIdentity} testID="stage-identity">STAGE</Text>
      </View>
      <View style={styles.stageViewport} testID="stage-viewport">
        {showGpu ? (
          <MotoliiGpuView
            accessible
            accessibilityLabel="Rerun Spatial Viewer Stage"
            createdItemId=""
            draggedItemId={draggedItemId}
            transformX={transform.x}
            transformY={transform.y}
            transformZ={transform.z}
            rotationX={transform.rotationX}
            rotationY={transform.rotationY}
            rotationZ={transform.rotationZ}
            onStageDrop={event =>
              onDrop(
                event.nativeEvent.x,
                event.nativeEvent.y,
                event.nativeEvent.canonicalX,
                event.nativeEvent.canonicalY,
              )
            }
            onStageTransform={() => {}}
            onHostTerminal={event => {
              const {accepted, message} = event.nativeEvent;
              applyNativeHostTerminal(accepted, message);
            }}
            style={styles.gpuStage}
            testID="rust-wgpu-stage"
          />
        ) : (
          <View style={styles.gpuStageOff}><Text style={styles.muted}>Native Stage unmounted</Text></View>
        )}
        <View pointerEvents="none" style={styles.frameGrid}>
          <Text style={styles.outputLabel}>OUTPUT FRAME</Text>
        </View>
      </View>
      <View style={styles.transport}>
        <Pressable
          accessibilityRole="button"
          accessibilityLabel="Skip to start"
          onPress={onSkipStart}
          testID="transport-skip-start">
          <Text style={styles.transportButton}>|‹</Text>
        </Pressable>
        <Pressable
          accessibilityRole="button"
          accessibilityLabel={playing ? 'Pause' : 'Play'}
          onPress={onTogglePlayback}
          testID="transport-play">
          <Text style={styles.transportButton}>{playing ? '⏸' : '▶'}</Text>
        </Pressable>
        <Pressable
          accessibilityRole="button"
          accessibilityLabel="Mark in"
          disabled={!primaryLayerId}
          onPress={onMarkIn}
          testID="transport-mark-in">
          <Text style={styles.transportButton}>I</Text>
        </Pressable>
        <Pressable
          accessibilityRole="button"
          accessibilityLabel="Mark out"
          disabled={!primaryLayerId}
          onPress={onMarkOut}
          testID="transport-mark-out">
          <Text style={styles.transportButton}>O</Text>
        </Pressable>
        <Pressable
          accessibilityRole="button"
          accessibilityLabel="Skip to end"
          onPress={onSkipEnd}
          testID="transport-skip-end">
          <Text style={styles.transportButton}>›|</Text>
        </Pressable>
        <Text style={styles.timecode}>{formatPlayhead(currentTime)}</Text>
        <Text style={styles.quality}>DRAFT · FP16 · 1/2</Text>
      </View>
    </View>
  );
}
