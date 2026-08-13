import React, {useRef} from 'react';
import {Pressable, Text, TextInput, View} from 'react-native';

import {type ShellKeyEvent} from './host';
import {styles} from './productStyles';

export const MacPressable = Pressable as React.ComponentType<
  React.ComponentProps<typeof Pressable> & {onDoubleClick?: () => void}
>;
export type DialKeyEvent = {
  nativeEvent: {key: string; shiftKey?: boolean};
  preventDefault: () => void;
};
export const DialSurface = Pressable as React.ComponentType<
  React.ComponentProps<typeof Pressable> & {
    onKeyDown?: (event: DialKeyEvent) => void;
    onTouchMoveCapture?: (event: {nativeEvent: {pageX: number}}) => void;
  }
>;
export const ShellView = View as React.ComponentType<
  React.ComponentProps<typeof View> & {
    onKeyDown?: (event: ShellKeyEvent) => void;
  }
>;

export function Splitter({
  label,
  orientation,
  onStart,
  onDelta,
  onNudge,
}: {
  label: string;
  orientation: 'vertical' | 'horizontal';
  onStart: () => void;
  onDelta: (delta: number) => void;
  onNudge: () => void;
}) {
  const pointerStart = useRef<number | null>(null);
  return (
    <Pressable
      accessibilityLabel={label}
      accessibilityRole="button"
      onPointerCancel={() => { pointerStart.current = null; }}
      onPointerDown={event => {
        pointerStart.current = orientation === 'vertical'
          ? event.nativeEvent.clientX
          : event.nativeEvent.clientY;
        onStart();
      }}
      onPointerMove={event => {
        if (pointerStart.current === null || event.nativeEvent.buttons === 0) {
          return;
        }
        const position = orientation === 'vertical'
          ? event.nativeEvent.clientX
          : event.nativeEvent.clientY;
        onDelta(position - pointerStart.current);
      }}
      onPointerUp={() => { pointerStart.current = null; }}
      onPress={onNudge}
      style={orientation === 'vertical' ? styles.vSplitter : styles.hSplitter}
    />
  );
}

export function PanelHeader({title, detail}: {title: string; detail?: string}) {
  return (
    <View style={styles.panelHeader}>
      <Text style={styles.panelTitle}>{title}</Text>
      {detail ? <Text style={styles.panelDetail}>{detail}</Text> : null}
    </View>
  );
}

export type ChromeModalKind = 'export' | 'settings';
export type ExportPhase = 'idle' | 'exporting' | 'complete' | 'failed';

export function ChromeModal({
  title,
  testID,
  onClose,
  children,
}: {
  title: string;
  testID: string;
  onClose: () => void;
  children: React.ReactNode;
}) {
  return (
    <View style={styles.chromeModalScrim} testID={testID}>
      <View style={styles.chromeModalCard}>
        <View style={styles.chromeModalHeader}>
          <Text style={styles.panelTitle}>{title}</Text>
          <Pressable
            accessibilityLabel={`Close ${title}`}
            onPress={onClose}
            style={styles.titleAction}
            testID={`${testID}-close`}>
            <Text style={styles.titleActionText}>Close</Text>
          </Pressable>
        </View>
        {children}
      </View>
    </View>
  );
}

export function ExportScreen({
  outputPath,
  onOutputPathChange,
  phase,
  statusText,
  onExport,
}: {
  outputPath: string;
  onOutputPathChange: (value: string) => void;
  phase: ExportPhase;
  statusText: string;
  onExport: () => void;
}) {
  const busy = phase === 'exporting';
  const canRun = !busy && outputPath.trim().length > 0;
  return (
    <View testID="export-screen">
      <View style={styles.parameterRow}>
        <Text style={styles.parameterLabel}>Output</Text>
        <TextInput
          editable={!busy}
          onChangeText={onOutputPathChange}
          placeholder="/path/to/output.mp4"
          placeholderTextColor="#858a8d"
          style={styles.parameterValue}
          testID="export-output-path"
          value={outputPath}
        />
      </View>
      <View style={styles.parameterRow}>
        <Text style={styles.parameterLabel}>Format</Text>
        <Text style={styles.parameterValue}>MP4 (H.264)</Text>
      </View>
      <View style={styles.chromeModalActions}>
        <Pressable
          accessibilityLabel="Run export"
          disabled={!canRun}
          onPress={onExport}
          style={[styles.titleAction, !canRun && styles.titleActionDisabled]}
          testID="export-run">
          <Text style={styles.titleActionText}>Export</Text>
        </Pressable>
      </View>
      <Text style={styles.chromeModalStatus} testID="export-status">
        {statusText}
      </Text>
    </View>
  );
}

export function SettingsScreen() {
  return (
    <View testID="settings-screen">
      <View style={styles.parameterRow} testID="settings-color-theme">
        <Text style={styles.parameterLabel}>Color theme</Text>
        <Text style={styles.muted}>Default</Text>
      </View>
      <View style={styles.parameterRow} testID="settings-keyboard">
        <Text style={styles.parameterLabel}>Keyboard</Text>
        <Text style={styles.muted}>Ableton Live</Text>
      </View>
    </View>
  );
}
