import React, {useCallback, useEffect, useMemo, useRef, useState} from 'react';
import {PanResponder, Pressable, ScrollView, Text, TextInput, View} from 'react-native';

import {panelRegistry} from './panels/registry';
import {DialSurface, PanelHeader} from './chrome';
import {
  cancelHostStageTransform,
  commitHostStageTransform,
  commitHostStageTransformGesture,
  dispatchHostIntent,
  exactParamKeyFor,
  previewHostStageTransform,
  type HostLayerSeat,
  type HostParamKey,
  type HostPositionKeyValue,
  type HostRationalTime,
  type StageTransform,
} from './host';
import {styles} from './productStyles';

export type RightPanel = 'INSPECTOR' | 'EXTENSIONS';

export function Inspector({width, transform, layerSeat, scrubFreeze, revision}: {width: number; transform: StageTransform; layerSeat: HostLayerSeat | null; scrubFreeze: boolean; revision: string | null}) {
  const [panel, setPanel] = useState<RightPanel>('INSPECTOR');
  const [extensionId, setExtensionId] = useState<string>(panelRegistry[0].id);
  const selectedLayerId = layerSeat?.primaryLayerId ?? null;
  const committed = useRef({
    x: transform.x,
    y: transform.y,
    rotationZ: transform.rotationZ,
    scaleX: transform.scaleX,
    scaleY: transform.scaleY,
  });
  const seen = useRef(
    `${transform.x},${transform.y},${transform.rotationZ},${transform.scaleX},${transform.scaleY}`,
  );
  const seenKey = `${transform.x},${transform.y},${transform.rotationZ},${transform.scaleX},${transform.scaleY}`;
  if (seen.current !== seenKey) {
    seen.current = seenKey;
    committed.current = {
      x: transform.x,
      y: transform.y,
      rotationZ: transform.rotationZ,
      scaleX: transform.scaleX,
      scaleY: transform.scaleY,
    };
  }
  // 凍結は(key, layer)のpairで行う。凍結中にprimaryが変わっても別layerへdispatchしない。
  const frozenExactSeat = useRef<{layerId: string; exactKey: HostPositionKeyValue} | null>(
    layerSeat?.exactKey && selectedLayerId
      ? {layerId: selectedLayerId, exactKey: layerSeat.exactKey}
      : null,
  );
  if (!scrubFreeze) {
    frozenExactSeat.current =
      layerSeat?.exactKey && selectedLayerId
        ? {layerId: selectedLayerId, exactKey: layerSeat.exactKey}
        : null;
  }
  const displayedExactSeat = scrubFreeze
    ? frozenExactSeat.current
    : layerSeat?.exactKey && selectedLayerId
      ? {layerId: selectedLayerId, exactKey: layerSeat.exactKey}
      : null;
  const liveExactParams =
    selectedLayerId && layerSeat
      ? {
          layerId: selectedLayerId,
          scale: exactParamKeyFor(layerSeat.paramKeys, layerSeat.currentTime, 'scale'),
          rotation: exactParamKeyFor(layerSeat.paramKeys, layerSeat.currentTime, 'rotation'),
          opacity: exactParamKeyFor(layerSeat.paramKeys, layerSeat.currentTime, 'opacity'),
        }
      : null;
  const frozenExactParams = useRef(liveExactParams);
  if (!scrubFreeze) {
    frozenExactParams.current = liveExactParams;
  }
  const displayedExactParams = scrubFreeze ? frozenExactParams.current : liveExactParams;
  const extension = panelRegistry.find(item => item.id === extensionId)!;

  return (
    <View style={[styles.inspector, {width}]} testID="inspector-surface">
      <PanelHeader
        title={panel === 'INSPECTOR' ? 'Inspector' : 'Extensions'}
        detail={selectedLayerId && layerSeat ? layerSeat.displayName : '未選択'}
      />
      <View style={styles.tabRow}>
        {(['INSPECTOR', 'EXTENSIONS'] as RightPanel[]).map(value => (
          <Pressable key={value} onPress={() => setPanel(value)} style={[styles.tab, panel === value && styles.tabActive]} testID={`right-panel-${value}`}>
            <Text style={styles.tabText}>{value === 'INSPECTOR' ? 'Effect' : 'Custom'}</Text>
          </Pressable>
        ))}
      </View>
      {panel === 'INSPECTOR' ? (
        <ScrollView disableScrollViewPanResponder>
          {selectedLayerId && layerSeat ? (
            <>
              <View style={styles.pathOperationSection} testID="inspector-layer-section">
                <Text style={styles.inspectorTitle}>{layerSeat.displayName}</Text>
                <Text style={styles.pathOperationDescription}>position keys: {layerSeat.positionKeyCount}</Text>
                {layerSeat.opacity != null ? (
                  <ParameterRow
                    label="Opacity"
                    value={String(layerSeat.opacity)}
                    testID="inspector-layer-opacity"
                    addKeyTestID="inspector-add-key-opacity"
                    onAddKey={() => {
                      dispatchHostIntent('add_param_key', {
                        target: selectedLayerId,
                        time: layerSeat.currentTime,
                        property: 'opacity',
                      });
                    }}
                    onChange={value => {
                      if (!selectedLayerId || !Number.isFinite(value)) {
                        return;
                      }
                      dispatchHostIntent('set_opacity', {
                        target: selectedLayerId,
                        value: Math.max(0, Math.min(1, value)),
                      });
                    }}
                    onDecrease={() => {
                      if (!selectedLayerId || layerSeat.opacity == null) {
                        return;
                      }
                      dispatchHostIntent('set_opacity', {
                        target: selectedLayerId,
                        value: Math.max(0, layerSeat.opacity - 0.05),
                      });
                    }}
                    onIncrease={() => {
                      if (!selectedLayerId || layerSeat.opacity == null) {
                        return;
                      }
                      dispatchHostIntent('set_opacity', {
                        target: selectedLayerId,
                        value: Math.min(1, layerSeat.opacity + 0.05),
                      });
                    }}
                  />
                ) : null}
                {displayedExactParams?.opacity?.value != null ? (
                  <ExactOnScalarParamKeyEditor
                    key={`${displayedExactParams.layerId}:opacity:${displayedExactParams.opacity.keyId}`}
                    primaryLayerId={displayedExactParams.layerId}
                    paramKey={displayedExactParams.opacity}
                    label="Opacity"
                    inputTestID="inspector-opacity-key"
                    accessibilityLabel="Opacity key"
                  />
                ) : null}
                {displayedExactSeat ? (
                  <ExactOnKeyValueEditor
                    key={`${displayedExactSeat.layerId}:${displayedExactSeat.exactKey.keyId}:${displayedExactSeat.exactKey.time.num}/${displayedExactSeat.exactKey.time.den}`}
                    primaryLayerId={displayedExactSeat.layerId}
                    keyId={displayedExactSeat.exactKey.keyId}
                    keyTime={displayedExactSeat.exactKey.time}
                    value={displayedExactSeat.exactKey.value}
                    interp={displayedExactSeat.exactKey.interp}
                  />
                ) : null}
                <View style={styles.pathOperationGrid}>
                  <Pressable
                    accessibilityRole="button"
                    accessibilityLabel="Delete clip"
                    onPress={() => {
                      dispatchHostIntent('delete_layer', {
                        target: selectedLayerId,
                      });
                    }}
                    style={styles.pathOperationButton}
                    testID="inspector-delete-clip">
                    <Text style={styles.pathOperationButtonText}>Delete</Text>
                  </Pressable>
                  <Pressable
                    accessibilityRole="button"
                    accessibilityLabel="Duplicate clip"
                    onPress={() => {
                      dispatchHostIntent('duplicate', {
                        target: selectedLayerId,
                      });
                    }}
                    style={styles.pathOperationButton}
                    testID="inspector-duplicate-clip">
                    <Text style={styles.pathOperationButtonText}>Duplicate</Text>
                  </Pressable>
                  <Pressable
                    accessibilityRole="button"
                    accessibilityLabel="Split clip"
                    onPress={() => {
                      dispatchHostIntent('split', {
                        target: selectedLayerId,
                        time: layerSeat.currentTime,
                      });
                    }}
                    style={styles.pathOperationButton}
                    testID="inspector-split-clip">
                    <Text style={styles.pathOperationButtonText}>Split</Text>
                  </Pressable>
                  <Pressable
                    accessibilityRole="button"
                    accessibilityLabel={layerSeat.visible ? 'Mute clip' : 'Unmute clip'}
                    onPress={() => {
                      dispatchHostIntent('mute', {target: selectedLayerId});
                    }}
                    style={styles.pathOperationButton}
                    testID="inspector-mute-clip">
                    <Text style={styles.pathOperationButtonText}>{layerSeat.visible ? 'Mute' : 'Unmute'}</Text>
                  </Pressable>
                  <Pressable
                    accessibilityRole="button"
                    accessibilityLabel={layerSeat.solo ? 'Unsolo clip' : 'Solo clip'}
                    onPress={() => {
                      dispatchHostIntent('solo', {target: selectedLayerId});
                    }}
                    style={styles.pathOperationButton}
                    testID="inspector-solo-clip">
                    <Text style={styles.pathOperationButtonText}>{layerSeat.solo ? 'Unsolo' : 'Solo'}</Text>
                  </Pressable>
                </View>
              </View>
              {layerSeat.sourceParams.length > 0 ? (
                <View style={styles.pathOperationSection} testID="inspector-source-params-section">
                  <Text style={styles.pathOperationTitle}>Source</Text>
                  {layerSeat.sourceParams.map(param =>
                    param.color ? (
                      <SourceColorParamEditor
                        key={`source:${param.param_id}`}
                        primaryLayerId={selectedLayerId}
                        paramId={param.param_id}
                        color={param.color}
                      />
                    ) : (
                      <SourceParamEditor
                        key={`source:${param.param_id}`}
                        primaryLayerId={selectedLayerId}
                        paramId={param.param_id}
                        value={param.value}
                      />
                    ),
                  )}
                </View>
              ) : null}
              {layerSeat.effects.length > 0 ? (
                <View style={styles.pathOperationSection} testID="inspector-effects-section">
                  <Text style={styles.pathOperationTitle}>Effects</Text>
                  {layerSeat.effects.map(effect => (
                    <View key={effect.effect_use_id} testID={`inspector-effect-${effect.effect_use_id}`}>
                      <Text style={styles.pathOperationDescription}>{effect.name}</Text>
                      {effect.params.map(param =>
                        param.color ? (
                          <EffectColorParamEditor
                            key={`${selectedLayerId}:${effect.effect_use_id}:${param.param_id}`}
                            primaryLayerId={selectedLayerId}
                            effectUseId={effect.effect_use_id}
                            paramId={param.param_id}
                            color={param.color}
                          />
                        ) : (
                          <EffectParamEditor
                            key={`${selectedLayerId}:${effect.effect_use_id}:${param.param_id}`}
                            primaryLayerId={selectedLayerId}
                            effectUseId={effect.effect_use_id}
                            paramId={param.param_id}
                            value={param.value}
                          />
                        ),
                      )}
                    </View>
                  ))}
                </View>
              ) : null}
              <View style={styles.transformSection} testID="stage-transform-projection">
                <Text style={styles.pathOperationTitle}>Transform</Text>
                <DialParameter label="Position X" value={transform.x} step={0.025} dragScale={0.01} decimals={3} addKeyTestID="inspector-add-key-position" onAddKey={() => {
                  // add_position_keyはtarget+time必須(rn_product_host 718-747)。Wake時刻は使わない。
                  dispatchHostIntent('add_position_key', {
                    target: selectedLayerId,
                    time: layerSeat.currentTime,
                  });
                }} onChange={value => {
                  if (!selectedLayerId || !layerSeat?.transform) {
                    return;
                  }
                  const delta = value - committed.current.x;
                  if (delta === 0 || !Number.isFinite(delta)) {
                    return;
                  }
                  if (dispatchHostIntent('move_layer_by', {target: selectedLayerId, delta: [delta, 0]})) {
                    committed.current.x = value;
                  }
                }} onDragPreview={value => {
                  if (!selectedLayerId || !layerSeat?.transform) {
                    return false;
                  }
                  const delta = value - committed.current.x;
                  return Number.isFinite(delta) && delta !== 0
                    ? previewHostStageTransform(selectedLayerId, revision, 0, delta, 0).accepted
                    : true;
                }} onDragCommit={value => {
                  if (!selectedLayerId || !layerSeat?.transform) {
                    return false;
                  }
                  const delta = value - committed.current.x;
                  if (!Number.isFinite(delta) || delta === 0) {
                    return true;
                  }
                  const accepted = commitHostStageTransformGesture(
                    selectedLayerId,
                    revision,
                    0,
                    delta,
                    0,
                  ).accepted;
                  if (accepted) {
                    committed.current.x = value;
                  }
                  return accepted;
                }} onDragCancel={() => cancelHostStageTransform().accepted} />
                <DialParameter label="Position Y" value={transform.y} step={0.025} dragScale={0.01} decimals={3} onChange={value => {
                  if (!selectedLayerId || !layerSeat?.transform) {
                    return;
                  }
                  const delta = value - committed.current.y;
                  if (delta === 0 || !Number.isFinite(delta)) {
                    return;
                  }
                  if (dispatchHostIntent('move_layer_by', {target: selectedLayerId, delta: [0, delta]})) {
                    committed.current.y = value;
                  }
                }} onDragPreview={value => {
                  if (!selectedLayerId || !layerSeat?.transform) {
                    return false;
                  }
                  const delta = value - committed.current.y;
                  return Number.isFinite(delta) && delta !== 0
                    ? previewHostStageTransform(selectedLayerId, revision, 0, 0, delta).accepted
                    : true;
                }} onDragCommit={value => {
                  if (!selectedLayerId || !layerSeat?.transform) {
                    return false;
                  }
                  const delta = value - committed.current.y;
                  if (!Number.isFinite(delta) || delta === 0) {
                    return true;
                  }
                  const accepted = commitHostStageTransformGesture(
                    selectedLayerId,
                    revision,
                    0,
                    0,
                    delta,
                  ).accepted;
                  if (accepted) {
                    committed.current.y = value;
                  }
                  return accepted;
                }} onDragCancel={() => cancelHostStageTransform().accepted} />
                <DialParameter label="Rotation Z" value={transform.rotationZ} unit="°" step={5} dragScale={1} decimals={1} addKeyTestID="inspector-add-key-rotation" onAddKey={() => {
                  dispatchHostIntent('add_param_key', {
                    target: selectedLayerId,
                    time: layerSeat.currentTime,
                    property: 'rotation',
                  });
                }} onChange={value => {
                  if (!selectedLayerId || !layerSeat?.transform) {
                    return;
                  }
                  const deltaDeg = value - committed.current.rotationZ;
                  if (deltaDeg === 0 || !Number.isFinite(deltaDeg)) {
                    return;
                  }
                  if (commitHostStageTransform(
                    selectedLayerId,
                    revision,
                    1,
                    deltaDeg * (Math.PI / 180),
                    0,
                  )) {
                    committed.current.rotationZ = value;
                  }
                }} onDragPreview={value => {
                  if (!selectedLayerId || !layerSeat?.transform) {
                    return false;
                  }
                  const deltaDeg = value - committed.current.rotationZ;
                  return Number.isFinite(deltaDeg) && deltaDeg !== 0
                    ? previewHostStageTransform(
                        selectedLayerId,
                        revision,
                        1,
                        deltaDeg * (Math.PI / 180),
                        0,
                      ).accepted
                    : true;
                }} onDragCommit={value => {
                  if (!selectedLayerId || !layerSeat?.transform) {
                    return false;
                  }
                  const deltaDeg = value - committed.current.rotationZ;
                  if (!Number.isFinite(deltaDeg) || deltaDeg === 0) {
                    return true;
                  }
                  const accepted = commitHostStageTransformGesture(
                    selectedLayerId,
                    revision,
                    1,
                    deltaDeg * (Math.PI / 180),
                    0,
                  ).accepted;
                  if (accepted) {
                    committed.current.rotationZ = value;
                  }
                  return accepted;
                }} onDragCancel={() => cancelHostStageTransform().accepted} />
                <DialParameter label="Scale X" value={transform.scaleX} step={0.025} dragScale={0.01} decimals={3} addKeyTestID="inspector-add-key-scale" onAddKey={() => {
                  dispatchHostIntent('add_param_key', {
                    target: selectedLayerId,
                    time: layerSeat.currentTime,
                    property: 'scale',
                  });
                }} onChange={value => {
                  if (!selectedLayerId || !layerSeat?.transform || committed.current.scaleX === 0) {
                    return;
                  }
                  if (value === committed.current.scaleX || !Number.isFinite(value)) {
                    return;
                  }
                  const factor = value / committed.current.scaleX;
                  if (!Number.isFinite(factor) || factor === 1) {
                    return;
                  }
                  if (commitHostStageTransform(selectedLayerId, revision, 2, factor, 1)) {
                    committed.current.scaleX = value;
                  }
                }} onDragPreview={value => {
                  if (!selectedLayerId || !layerSeat?.transform || committed.current.scaleX === 0) {
                    return false;
                  }
                  const factor = value / committed.current.scaleX;
                  return Number.isFinite(factor) && factor !== 1
                    ? previewHostStageTransform(selectedLayerId, revision, 2, factor, 1).accepted
                    : true;
                }} onDragCommit={value => {
                  if (!selectedLayerId || !layerSeat?.transform || committed.current.scaleX === 0) {
                    return false;
                  }
                  const factor = value / committed.current.scaleX;
                  if (!Number.isFinite(factor) || factor === 1) {
                    return true;
                  }
                  const accepted = commitHostStageTransformGesture(
                    selectedLayerId,
                    revision,
                    2,
                    factor,
                    1,
                  ).accepted;
                  if (accepted) {
                    committed.current.scaleX = value;
                  }
                  return accepted;
                }} onDragCancel={() => cancelHostStageTransform().accepted} />
                <DialParameter label="Scale Y" value={transform.scaleY} step={0.025} dragScale={0.01} decimals={3} onChange={value => {
                  if (!selectedLayerId || !layerSeat?.transform || committed.current.scaleY === 0) {
                    return;
                  }
                  if (value === committed.current.scaleY || !Number.isFinite(value)) {
                    return;
                  }
                  const factor = value / committed.current.scaleY;
                  if (!Number.isFinite(factor) || factor === 1) {
                    return;
                  }
                  if (commitHostStageTransform(selectedLayerId, revision, 2, 1, factor)) {
                    committed.current.scaleY = value;
                  }
                }} onDragPreview={value => {
                  if (!selectedLayerId || !layerSeat?.transform || committed.current.scaleY === 0) {
                    return false;
                  }
                  const factor = value / committed.current.scaleY;
                  return Number.isFinite(factor) && factor !== 1
                    ? previewHostStageTransform(selectedLayerId, revision, 2, 1, factor).accepted
                    : true;
                }} onDragCommit={value => {
                  if (!selectedLayerId || !layerSeat?.transform || committed.current.scaleY === 0) {
                    return false;
                  }
                  const factor = value / committed.current.scaleY;
                  if (!Number.isFinite(factor) || factor === 1) {
                    return true;
                  }
                  const accepted = commitHostStageTransformGesture(
                    selectedLayerId,
                    revision,
                    2,
                    1,
                    factor,
                  ).accepted;
                  if (accepted) {
                    committed.current.scaleY = value;
                  }
                  return accepted;
                }} onDragCancel={() => cancelHostStageTransform().accepted} />
                {displayedExactParams?.rotation?.value != null ? (
                  <ExactOnScalarParamKeyEditor
                    key={`${displayedExactParams.layerId}:rotation:${displayedExactParams.rotation.keyId}`}
                    primaryLayerId={displayedExactParams.layerId}
                    paramKey={displayedExactParams.rotation}
                    label="Rotation Z"
                    unit="°"
                    inputTestID="inspector-rotation-key"
                    accessibilityLabel="Rotation key"
                    display="degrees"
                  />
                ) : null}
                {displayedExactParams?.scale?.vec ? (
                  <ExactOnScaleKeyEditor
                    key={`${displayedExactParams.layerId}:scale:${displayedExactParams.scale.keyId}`}
                    primaryLayerId={displayedExactParams.layerId}
                    paramKey={displayedExactParams.scale}
                  />
                ) : null}
              </View>
            </>
          ) : (
            <View style={styles.emptyPanel} testID="inspector-empty">
              <Text style={styles.emptyTitle}>未選択</Text>
            </View>
          )}
        </ScrollView>
      ) : (
        <View style={styles.extensionBody}>
          <View style={styles.extensionTabs}>
            {panelRegistry.map(item => (
              <Pressable key={item.id} onPress={() => setExtensionId(item.id)} style={[styles.extensionTab, extensionId === item.id && styles.iconButtonActive]}>
                <Text style={styles.tabText}>{item.title}</Text>
              </Pressable>
            ))}
          </View>
          <extension.Component />
        </View>
      )}
    </View>
  );
}

export function ParameterRow({label, value, onChange, onDecrease, onIncrease, testID, onAddKey, addKeyTestID}: {label: string; value: string; onChange?: (value: number) => void; onDecrease?: () => void; onIncrease?: () => void; testID?: string; onAddKey?: () => void; addKeyTestID?: string}) {
  const numeric = Number(value);
  const decrease = onDecrease ?? (onChange && Number.isFinite(numeric) ? () => onChange(numeric - 0.05) : undefined);
  const increase = onIncrease ?? (onChange && Number.isFinite(numeric) ? () => onChange(numeric + 0.05) : undefined);
  return (
    <View style={styles.parameterRow} testID={testID}>
      <Text style={styles.parameterLabel}>{label}</Text>
      {onAddKey && addKeyTestID ? (
        <Pressable
          accessibilityRole="button"
          accessibilityLabel={`Add ${label} key`}
          onPress={onAddKey}
          style={styles.stepButton}
          testID={addKeyTestID}>
          <Text style={styles.stepText}>◆</Text>
        </Pressable>
      ) : null}
      {decrease ? <Pressable accessibilityLabel={`${label} decrease`} onPress={decrease} style={styles.stepButton}><Text style={styles.stepText}>−</Text></Pressable> : null}
      <Text numberOfLines={1} style={styles.parameterValue}>{value}</Text>
      {increase ? <Pressable accessibilityLabel={`${label} increase`} onPress={increase} style={styles.stepButton}><Text style={styles.stepText}>＋</Text></Pressable> : null}
    </View>
  );
}

const COLOR_CHANNELS = ['r', 'g', 'b', 'a'] as const;

/** source/effect Color を別widgetにすると commit 形が分岐するため共有する。 */
function ColorRgbaEditor({
  color,
  label,
  wrapperTestId,
  rowTestIdPrefix,
  inputTestIdPrefix,
  accessibilityPrefix,
  onPreview,
  onCommit,
}: {
  color: [number, number, number, number];
  label: string;
  wrapperTestId: string;
  rowTestIdPrefix: string;
  inputTestIdPrefix: string;
  accessibilityPrefix: string;
  onPreview: (color: [number, number, number, number]) => void;
  onCommit: (color: [number, number, number, number]) => boolean;
}) {
  const [drafts, setDrafts] = useState(color.map(String));
  const live = useRef(color);
  const editing = useRef([false, false, false, false]);

  useEffect(() => {
    live.current = color;
    setDrafts(prev => prev.map((draft, index) => (editing.current[index] ? draft : String(color[index]))));
  }, [color]);

  const commitChannel = (index: number) => {
    if (!editing.current[index]) {
      return;
    }
    editing.current[index] = false;
    const draft = drafts[index] ?? '';
    if (draft.trim().length === 0) {
      setDrafts(prev => {
        const next = [...prev];
        next[index] = String(live.current[index]);
        return next;
      });
      return;
    }
    const parsed = Number(draft);
    if (!Number.isFinite(parsed)) {
      setDrafts(prev => {
        const next = [...prev];
        next[index] = String(live.current[index]);
        return next;
      });
      return;
    }
    const next: [number, number, number, number] = [...live.current];
    next[index] = parsed;
    if (!onCommit(next)) {
      setDrafts(prev => {
        const nextDrafts = [...prev];
        nextDrafts[index] = String(live.current[index]);
        return nextDrafts;
      });
      return;
    }
    live.current = next;
    setDrafts(prev => prev.map((draftValue, draftIndex) => (
      editing.current[draftIndex] ? draftValue : String(next[draftIndex])
    )));
  };

  return (
    <View testID={wrapperTestId}>
      <Text style={styles.pathOperationDescription}>{label}</Text>
      {COLOR_CHANNELS.map((channel, index) => (
        <View key={channel} style={styles.parameterRow} testID={`${rowTestIdPrefix}-${channel}`}>
          <Text style={styles.parameterLabel}>{channel.toUpperCase()}</Text>
          <TextInput
            accessibilityLabel={`${accessibilityPrefix} ${channel}`}
            keyboardType="decimal-pad"
            onBlur={() => commitChannel(index)}
            onChangeText={text => {
              editing.current[index] = true;
              setDrafts(prev => {
                const nextDrafts = [...prev];
                nextDrafts[index] = text;
                return nextDrafts;
              });
              if (text.trim().length === 0) {
                return;
              }
              const parsed = Number(text);
              if (!Number.isFinite(parsed)) {
                return;
              }
              const next: [number, number, number, number] = [...live.current];
              next[index] = parsed;
              onPreview(next);
            }}
            onFocus={() => {
              editing.current[index] = true;
            }}
            onSubmitEditing={() => commitChannel(index)}
            selectTextOnFocus
            style={styles.parameterValue}
            testID={`${inputTestIdPrefix}-${channel}`}
            value={drafts[index]}
          />
        </View>
      ))}
    </View>
  );
}

/** vism source Color Const: commit 時1回 set_source_param。f64 経路は SourceParamEditor。 */
export function SourceColorParamEditor({
  primaryLayerId,
  paramId,
  color,
}: {
  primaryLayerId: string;
  paramId: string;
  color: [number, number, number, number];
}) {
  return (
    <ColorRgbaEditor
      accessibilityPrefix={`Source ${paramId}`}
      color={color}
      inputTestIdPrefix={`inspector-source-color-input-${paramId}`}
      label={paramId}
      onCommit={next => dispatchHostIntent('set_source_param', {
        target: primaryLayerId,
        param_id: paramId,
        color: next,
      })}
      onPreview={next => {
        dispatchHostIntent('preview_source_param', {
          target: primaryLayerId,
          param_id: paramId,
          color: next,
        });
      }}
      rowTestIdPrefix={`inspector-source-color-${paramId}`}
      wrapperTestId={`inspector-source-param-${paramId}`}
    />
  );
}

/** Color Const に f64 value を付けると runtime が no-op にする。 */
function EffectColorParamEditor({
  primaryLayerId,
  effectUseId,
  paramId,
  color,
}: {
  primaryLayerId: string;
  effectUseId: string;
  paramId: string;
  color: [number, number, number, number];
}) {
  return (
    <ColorRgbaEditor
      accessibilityPrefix={`Effect ${paramId}`}
      color={color}
      inputTestIdPrefix={`inspector-effect-color-input-${effectUseId}-${paramId}`}
      label={paramId}
      onCommit={next => dispatchHostIntent('set_effect_param', {
        target: primaryLayerId,
        effect_use_id: effectUseId,
        param_id: paramId,
        color: next,
      })}
      onPreview={next => {
        dispatchHostIntent('preview_effect_param', {
          target: primaryLayerId,
          effect_use_id: effectUseId,
          param_id: paramId,
          color: next,
        });
      }}
      rowTestIdPrefix={`inspector-effect-color-${effectUseId}-${paramId}`}
      wrapperTestId={`inspector-effect-param-${effectUseId}-${paramId}`}
    />
  );
}

/** vism source f64 param: commit 時1回 set_source_param。型天井は D2 DocParam 側。 */
export function SourceParamEditor({
  primaryLayerId,
  paramId,
  value,
}: {
  primaryLayerId: string;
  paramId: string;
  value: number;
}) {
  const [draft, setDraft] = useState(String(value));
  const live = useRef(value);
  const editing = useRef(false);

  useEffect(() => {
    live.current = value;
    if (!editing.current) {
      setDraft(String(value));
    }
  }, [value]);

  const commit = () => {
    if (!editing.current) {
      return;
    }
    editing.current = false;
    if (draft.trim().length === 0) {
      setDraft(String(live.current));
      return;
    }
    const parsed = Number(draft);
    if (!Number.isFinite(parsed)) {
      setDraft(String(live.current));
      return;
    }
    const accepted = dispatchHostIntent('set_source_param', {
      target: primaryLayerId,
      param_id: paramId,
      value: parsed,
    });
    if (!accepted) {
      setDraft(String(live.current));
      return;
    }
    live.current = parsed;
    setDraft(String(parsed));
  };

  return (
    <View style={styles.parameterRow} testID={`inspector-source-param-${paramId}`}>
      <Text style={styles.parameterLabel}>{paramId}</Text>
      <TextInput
        accessibilityLabel={`Source ${paramId}`}
        keyboardType="decimal-pad"
        onBlur={commit}
        onChangeText={text => {
          editing.current = true;
          setDraft(text);
          if (text.trim().length === 0) {
            return;
          }
          const parsed = Number(text);
          if (!Number.isFinite(parsed)) {
            return;
          }
          dispatchHostIntent('preview_source_param', {
            target: primaryLayerId,
            param_id: paramId,
            value: parsed,
          });
        }}
        onFocus={() => {
          editing.current = true;
        }}
        onSubmitEditing={commit}
        selectTextOnFocus
        style={styles.parameterValue}
        testID={`inspector-source-param-input-${paramId}`}
        value={draft}
      />
    </View>
  );
}

/** effect f64 param: commit 時1回 set_effect_param。二重送信防止と巻き戻しは exact-on-key と同作法。 */
export function EffectParamEditor({
  primaryLayerId,
  effectUseId,
  paramId,
  value,
}: {
  primaryLayerId: string;
  effectUseId: string;
  paramId: string;
  value: number;
}) {
  const [draft, setDraft] = useState(String(value));
  const live = useRef(value);
  const editing = useRef(false);

  useEffect(() => {
    live.current = value;
    if (!editing.current) {
      setDraft(String(value));
    }
  }, [value]);

  const commit = () => {
    if (!editing.current) {
      return;
    }
    editing.current = false;
    if (draft.trim().length === 0) {
      setDraft(String(live.current));
      return;
    }
    const parsed = Number(draft);
    if (!Number.isFinite(parsed)) {
      setDraft(String(live.current));
      return;
    }
    const accepted = dispatchHostIntent('set_effect_param', {
      target: primaryLayerId,
      effect_use_id: effectUseId,
      param_id: paramId,
      value: parsed,
    });
    if (!accepted) {
      setDraft(String(live.current));
      return;
    }
    live.current = parsed;
    setDraft(String(parsed));
  };

  return (
    <View style={styles.parameterRow} testID={`inspector-effect-param-${effectUseId}-${paramId}`}>
      <Text style={styles.parameterLabel}>{paramId}</Text>
      <TextInput
        accessibilityLabel={`Effect ${paramId}`}
        keyboardType="decimal-pad"
        onBlur={commit}
        onChangeText={text => {
          editing.current = true;
          setDraft(text);
          if (text.trim().length === 0) {
            return;
          }
          const parsed = Number(text);
          if (!Number.isFinite(parsed)) {
            return;
          }
          dispatchHostIntent('preview_effect_param', {
            target: primaryLayerId,
            effect_use_id: effectUseId,
            param_id: paramId,
            value: parsed,
          });
        }}
        onFocus={() => {
          editing.current = true;
        }}
        onSubmitEditing={commit}
        selectTextOnFocus
        style={styles.parameterValue}
        testID={`inspector-effect-param-input-${effectUseId}-${paramId}`}
        value={draft}
      />
    </View>
  );
}

function ExactOnScaleKeyEditor({
  primaryLayerId,
  paramKey,
}: {
  primaryLayerId: string;
  paramKey: HostParamKey;
}) {
  const value = paramKey.vec ?? [1, 1];
  const [draftX, setDraftX] = useState(String(value[0]));
  const [draftY, setDraftY] = useState(String(value[1]));
  const live = useRef(value);
  const editingX = useRef(false);
  const editingY = useRef(false);
  const valueX = value[0];
  const valueY = value[1];

  useEffect(() => {
    live.current = [valueX, valueY];
    if (!editingX.current) {
      setDraftX(String(valueX));
    }
    if (!editingY.current) {
      setDraftY(String(valueY));
    }
  }, [valueX, valueY]);

  const commitAxis = (axis: 0 | 1, draft: string) => {
    const isEditing = axis === 0 ? editingX : editingY;
    if (!isEditing.current) {
      return;
    }
    isEditing.current = false;
    const revert = () => {
      if (axis === 0) {
        setDraftX(String(live.current[0]));
      } else {
        setDraftY(String(live.current[1]));
      }
    };
    if (draft.trim().length === 0) {
      revert();
      return;
    }
    const parsed = Number(draft);
    if (!Number.isFinite(parsed)) {
      revert();
      return;
    }
    const next: [number, number] = [live.current[0], live.current[1]];
    next[axis] = parsed;
    const accepted = dispatchHostIntent('set_param_key_value', {
      target: primaryLayerId,
      key_id: paramKey.keyId,
      property: 'scale',
      new: next,
    });
    if (!accepted) {
      revert();
      return;
    }
    live.current = next;
    if (axis === 0) {
      setDraftX(String(next[0]));
      if (!editingY.current) {
        setDraftY(String(next[1]));
      }
    } else {
      setDraftY(String(next[1]));
      if (!editingX.current) {
        setDraftX(String(next[0]));
      }
    }
  };

  return (
    <>
      <View style={styles.parameterRow} testID="inspector-scale-key-x-row">
        <Text style={styles.parameterLabel}>Scale X</Text>
        <TextInput
          accessibilityLabel="Scale key X"
          keyboardType="decimal-pad"
          onBlur={() => commitAxis(0, draftX)}
          onChangeText={text => {
            editingX.current = true;
            setDraftX(text);
          }}
          onFocus={() => {
            editingX.current = true;
          }}
          onSubmitEditing={() => commitAxis(0, draftX)}
          selectTextOnFocus
          style={styles.parameterValue}
          testID="inspector-scale-key-x"
          value={draftX}
        />
      </View>
      <View style={styles.parameterRow} testID="inspector-scale-key-y-row">
        <Text style={styles.parameterLabel}>Scale Y</Text>
        <TextInput
          accessibilityLabel="Scale key Y"
          keyboardType="decimal-pad"
          onBlur={() => commitAxis(1, draftY)}
          onChangeText={text => {
            editingY.current = true;
            setDraftY(text);
          }}
          onFocus={() => {
            editingY.current = true;
          }}
          onSubmitEditing={() => commitAxis(1, draftY)}
          selectTextOnFocus
          style={styles.parameterValue}
          testID="inspector-scale-key-y"
          value={draftY}
        />
      </View>
    </>
  );
}

function ExactOnScalarParamKeyEditor({
  primaryLayerId,
  paramKey,
  label,
  unit = '',
  inputTestID,
  accessibilityLabel,
  display,
}: {
  primaryLayerId: string;
  paramKey: HostParamKey;
  label: string;
  unit?: string;
  inputTestID: string;
  accessibilityLabel: string;
  display?: 'degrees';
}) {
  const toDisplay = (value: number) =>
    display === 'degrees' ? value * (180 / Math.PI) : value;
  const fromDisplay = (value: number) =>
    display === 'degrees' ? value * (Math.PI / 180) : value;
  const stored = paramKey.value ?? 0;
  const shown = toDisplay(stored);
  const [draft, setDraft] = useState(String(shown));
  const live = useRef(stored);
  const editing = useRef(false);

  useEffect(() => {
    live.current = stored;
    if (!editing.current) {
      setDraft(String(display === 'degrees' ? stored * (180 / Math.PI) : stored));
    }
  }, [stored, display]);

  const commit = (text: string) => {
    if (!editing.current) {
      return;
    }
    editing.current = false;
    const revert = () => {
      setDraft(String(toDisplay(live.current)));
    };
    if (text.trim().length === 0) {
      revert();
      return;
    }
    const parsed = Number(text);
    if (!Number.isFinite(parsed)) {
      revert();
      return;
    }
    const next = fromDisplay(parsed);
    if (!Number.isFinite(next)) {
      revert();
      return;
    }
    const accepted = dispatchHostIntent('set_param_key_value', {
      target: primaryLayerId,
      key_id: paramKey.keyId,
      property: paramKey.property,
      value: next,
    });
    if (!accepted) {
      revert();
      return;
    }
    live.current = next;
    setDraft(String(toDisplay(next)));
  };

  return (
    <View style={styles.parameterRow} testID={`${inputTestID}-row`}>
      <Text style={styles.parameterLabel}>
        {label}
        {unit}
      </Text>
      <TextInput
        accessibilityLabel={accessibilityLabel}
        keyboardType="decimal-pad"
        onBlur={() => commit(draft)}
        onChangeText={text => {
          editing.current = true;
          setDraft(text);
        }}
        onFocus={() => {
          editing.current = true;
        }}
        onSubmitEditing={() => commit(draft)}
        selectTextOnFocus
        style={styles.parameterValue}
        testID={inputTestID}
        value={draft}
      />
    </View>
  );
}

/** U4b-0V: exact-on-key の時だけ X/Y を commit 時1回で送る。Dialは使わない。 */
export function ExactOnKeyValueEditor({
  primaryLayerId,
  keyId,
  keyTime,
  value,
  interp,
}: {
  primaryLayerId: string;
  keyId: string;
  keyTime: HostRationalTime;
  value: [number, number];
  interp: string | null;
}) {
  const [draftX, setDraftX] = useState(String(value[0]));
  const [draftY, setDraftY] = useState(String(value[1]));
  const live = useRef(value);
  const editingX = useRef(false);
  const editingY = useRef(false);
  const valueX = value[0];
  const valueY = value[1];

  useEffect(() => {
    live.current = [valueX, valueY];
    if (!editingX.current) {
      setDraftX(String(valueX));
    }
    if (!editingY.current) {
      setDraftY(String(valueY));
    }
  }, [valueX, valueY]);

  const commitAxis = (axis: 0 | 1, draft: string) => {
    const isEditing = axis === 0 ? editingX : editingY;
    if (!isEditing.current) {
      return;
    }
    isEditing.current = false;

    if (draft.trim().length === 0) {
      if (axis === 0) {
        setDraftX(String(live.current[0]));
      } else {
        setDraftY(String(live.current[1]));
      }
      return;
    }

    const parsed = Number(draft);
    if (!Number.isFinite(parsed)) {
      if (axis === 0) {
        setDraftX(String(live.current[0]));
      } else {
        setDraftY(String(live.current[1]));
      }
      return;
    }
    const next: [number, number] = [live.current[0], live.current[1]];
    next[axis] = parsed;
    // set_position_key_value: target/time/new (rn_product_host 743-849)
    const accepted = dispatchHostIntent('set_position_key_value', {
      target: primaryLayerId,
      time: keyTime,
      new: next,
    });
    if (!accepted) {
      if (axis === 0) {
        setDraftX(String(live.current[0]));
      } else {
        setDraftY(String(live.current[1]));
      }
      return;
    }
    live.current = next;
    // 他軸が編集中ならその draft を上書きしない。
    if (axis === 0) {
      setDraftX(String(next[0]));
      if (!editingY.current) {
        setDraftY(String(next[1]));
      }
    } else {
      setDraftY(String(next[1]));
      if (!editingX.current) {
        setDraftX(String(next[0]));
      }
    }
  };

  return (
    <>
      <View style={styles.parameterRow} testID="inspector-position-key-x-row">
        <Text style={styles.parameterLabel}>X</Text>
        <TextInput
          accessibilityLabel="Position key X"
          keyboardType="decimal-pad"
          onBlur={() => commitAxis(0, draftX)}
        onChangeText={text => {
          editingX.current = true;
          setDraftX(text);
        }}
        onFocus={() => {
          editingX.current = true;
        }}
        onSubmitEditing={() => commitAxis(0, draftX)}
        selectTextOnFocus
        style={styles.parameterValue}
        testID="inspector-position-key-x"
        value={draftX}
        />
      </View>
      <View style={styles.parameterRow} testID="inspector-position-key-y-row">
        <Text style={styles.parameterLabel}>Y</Text>
        <TextInput
          accessibilityLabel="Position key Y"
          keyboardType="decimal-pad"
          onBlur={() => commitAxis(1, draftY)}
        onChangeText={text => {
          editingY.current = true;
          setDraftY(text);
        }}
        onFocus={() => {
          editingY.current = true;
        }}
        onSubmitEditing={() => commitAxis(1, draftY)}
        selectTextOnFocus
        style={styles.parameterValue}
        testID="inspector-position-key-y"
          value={draftY}
        />
      </View>
      {interp === 'Hold' || interp === 'Linear' || interp == null ? (
        <View style={styles.pathOperationGrid}>
          <Pressable
            accessibilityRole="button"
            accessibilityLabel="Hold interpolation"
            onPress={() => {
              dispatchHostIntent('set_position_key_interp', {
                target: primaryLayerId,
                time: keyTime,
                interp: 'Hold',
              });
            }}
            style={styles.pathOperationButton}
            testID="inspector-key-interp-hold">
            <Text style={styles.pathOperationButtonText}>{interp === 'Hold' ? 'Hold ✓' : 'Hold'}</Text>
          </Pressable>
          <Pressable
            accessibilityRole="button"
            accessibilityLabel="Linear interpolation"
            onPress={() => {
              dispatchHostIntent('set_position_key_interp', {
                target: primaryLayerId,
                time: keyTime,
                interp: 'Linear',
              });
            }}
            style={styles.pathOperationButton}
            testID="inspector-key-interp-linear">
            <Text style={styles.pathOperationButtonText}>{interp === 'Linear' ? 'Linear ✓' : 'Linear'}</Text>
          </Pressable>
        </View>
      ) : null}
      <View style={styles.pathOperationGrid}>
        <Pressable
          accessibilityRole="button"
          accessibilityLabel="Remove position key"
          onPress={() => {
            dispatchHostIntent('remove_position_key', {
              target: primaryLayerId,
              key_id: keyId,
            });
          }}
          style={styles.pathOperationButton}
          testID="inspector-remove-position-key">
          <Text style={styles.pathOperationButtonText}>Remove Key</Text>
        </Pressable>
      </View>
    </>
  );
}

export function DialParameter({label, value, unit = '', step = 1, dragScale = 1, decimals = 0, min, max, onChange, onDragPreview, onDragCommit, onDragCancel, onAddKey, addKeyTestID}: {label: string; value: number; unit?: string; step?: number; dragScale?: number; decimals?: number; min?: number; max?: number; onChange: (value: number) => void; onDragPreview?: (value: number) => boolean; onDragCommit?: (value: number) => boolean; onDragCancel?: () => boolean; onAddKey?: () => void; addKeyTestID?: string}) {
  const [draft, setDraft] = useState(formatDialValue(value, decimals));
  const [dragging, setDragging] = useState(false);
  const pointerStart = useRef<{
    pageX: number;
    value: number;
    previewAccepted: boolean;
    previewed: boolean;
    previewValue: number;
  } | null>(null);
  const editing = useRef(false);

  useEffect(() => {
    if (!editing.current && !pointerStart.current) {
      setDraft(formatDialValue(value, decimals));
    }
  }, [decimals, value]);

  const normalizedValue = useCallback((next: number, snap = false) => {
    const stepped = snap ? Math.round(next / step) * step : next;
    const bounded = Math.max(min ?? -Infinity, Math.min(max ?? Infinity, stepped));
    return Number(bounded.toFixed(Math.max(decimals, 6)));
  }, [decimals, max, min, step]);

  const finishDrag = useCallback((disposition: 'commit' | 'cancel') => {
    const start = pointerStart.current;
    pointerStart.current = null;
    setDragging(false);
    if (!start) {
      return;
    }
    if (disposition === 'cancel' || start.previewValue === start.value) {
      if (onDragCancel) {
        if (start.previewed && start.previewAccepted) {
          onDragCancel();
        }
      } else if (disposition === 'cancel' && start.previewValue !== start.value) {
        onChange(start.value);
      }
      setDraft(formatDialValue(start.value, decimals));
      return;
    }
    if (!start.previewAccepted) {
      setDraft(formatDialValue(start.value, decimals));
      return;
    }
    if (onDragCommit && !onDragCommit(start.previewValue)) {
      setDraft(formatDialValue(start.value, decimals));
    }
  }, [decimals, onChange, onDragCancel, onDragCommit]);

  const previewDrag = useCallback((next: number) => {
    const start = pointerStart.current;
    if (!start || next === start.previewValue) {
      return;
    }
    start.previewValue = next;
    setDraft(formatDialValue(next, decimals));
    if (onDragPreview) {
      start.previewed = true;
      start.previewAccepted = onDragPreview(next);
    } else {
      onChange(next);
      start.previewAccepted = true;
    }
  }, [decimals, onChange, onDragPreview]);

  const panResponder = useMemo(() => PanResponder.create({
    onStartShouldSetPanResponder: () => true,
    onMoveShouldSetPanResponder: (_, gestureState) => (
      Math.abs(gestureState.dx) > Math.abs(gestureState.dy) && Math.abs(gestureState.dx) > 1
    ),
    onMoveShouldSetPanResponderCapture: (_, gestureState) => (
      Math.abs(gestureState.dx) > Math.abs(gestureState.dy) && Math.abs(gestureState.dx) > 1
    ),
    onPanResponderGrant: () => {
      pointerStart.current ??= {
        pageX: 0,
        value,
        previewAccepted: true,
        previewed: false,
        previewValue: value,
      };
      editing.current = false;
      setDragging(true);
    },
    onPanResponderMove: (_, gestureState) => {
      const start = pointerStart.current;
      if (!start) return;
      const next = normalizedValue(start.value + gestureState.dx * dragScale);
      previewDrag(next);
    },
    onPanResponderRelease: () => {
      finishDrag('commit');
    },
    onPanResponderTerminate: () => {
      finishDrag('cancel');
    },
  }), [dragScale, finishDrag, normalizedValue, previewDrag, value]);

  const dialShift = ((value * 2) % 50 + 50) % 50;

  const commitDraft = () => {
    const parsed = Number(draft.trim());
    if (Number.isFinite(parsed)) {
      const next = normalizedValue(parsed, false);
      onChange(next);
      setDraft(formatDialValue(next, decimals));
    } else {
      setDraft(formatDialValue(value, decimals));
    }
    editing.current = false;
  };

  const applyTouchMove = (pageX: number) => {
    const start = pointerStart.current;
    if (!start) return;
    if (!Number.isFinite(pageX)) {
      return;
    }
    const delta = pageX - start.pageX;
    const next = normalizedValue(start.value + delta * dragScale);
    previewDrag(next);
  };

  return (
    <View style={styles.parameterRow}>
      <Text style={styles.parameterLabel}>{label}</Text>
      {onAddKey && addKeyTestID ? (
        <Pressable
          accessibilityRole="button"
          accessibilityLabel={`Add ${label} key`}
          onPress={onAddKey}
          style={styles.stepButton}
          testID={addKeyTestID}>
          <Text style={styles.stepText}>◆</Text>
        </Pressable>
      ) : null}
      <DialSurface
        {...panResponder.panHandlers}
        accessible
        accessibilityRole="adjustable"
        accessibilityLabel={`${label} dial`}
        accessibilityHint="Drag horizontally across the infinite ticks to change the value"
        style={[styles.dial, dragging && styles.dialDragging]}
        onTouchStart={event => {
          pointerStart.current = {
            pageX: event.nativeEvent.pageX,
            value,
            previewAccepted: true,
            previewed: false,
            previewValue: value,
          };
          editing.current = false;
          setDragging(true);
        }}
        onTouchMove={event => {
          applyTouchMove(event.nativeEvent.pageX);
        }}
        onTouchMoveCapture={event => {
          applyTouchMove(event.nativeEvent.pageX);
        }}
        onTouchEnd={() => {
          finishDrag('commit');
        }}
        onTouchCancel={() => {
          finishDrag('cancel');
        }}
        onKeyDown={event => {
          if (!['ArrowLeft', 'ArrowRight'].includes(event.nativeEvent.key)) return;
          event.preventDefault();
          const direction = event.nativeEvent.key === 'ArrowRight' ? 1 : -1;
          const multiplier = event.nativeEvent.shiftKey ? 10 : 1;
          const next = normalizedValue(value + direction * step * multiplier);
          setDraft(formatDialValue(next, decimals));
          onChange(next);
        }}>
        <View pointerEvents="none" style={styles.dialTicks}>
          <View pointerEvents="none" style={[styles.dialTickStrip, {transform: [{translateX: -dialShift}]}]}>
            {Array.from({length: 81}, (_, index) => (
              <View key={`${label}-tick-${index}`} style={styles.dialTickCell}>
                <View style={[styles.dialTick, index % 5 === 0 && styles.dialTickMajor]} />
              </View>
            ))}
          </View>
          <View style={styles.dialPointer} />
        </View>
        <TextInput
          accessibilityLabel={`${label} value`}
          keyboardType="decimal-pad"
          onBlur={commitDraft}
          onChangeText={text => {
            editing.current = true;
            setDraft(text);
          }}
          onFocus={() => { editing.current = true; }}
          onSubmitEditing={commitDraft}
          selectTextOnFocus
          style={[styles.dialValue, unit ? styles.dialValueWithUnit : null]}
          value={draft}
        />
        {unit ? <Text pointerEvents="none" style={styles.dialUnit}>{unit}</Text> : null}
      </DialSurface>
    </View>
  );
}

export const formatDialValue = (value: number, decimals: number) => value.toFixed(decimals);
