import type {TurboModule} from 'react-native';
import {TurboModuleRegistry} from 'react-native';

export interface Spec extends TurboModule {
  dispatchIntent(intentJson: string): string;
  readSnapshot(): string;
  isTimelineInteracting(): boolean;
  fitStageView(): boolean;
  stageViewOneToOne(): boolean;
  /** 既存 gizmo commit の alias。kind 1=RotateZ(a rad), 2=Scale([a,b]). */
  commitStageTransform(
    target: string,
    revision: string,
    kind: number,
    a: number,
    b: number,
  ): boolean;
  previewStageTransform(
    target: string,
    revision: string,
    kind: number,
    a: number,
    b: number,
  ): string;
  commitStageTransformGesture(
    target: string,
    revision: string,
    kind: number,
    a: number,
    b: number,
  ): string;
  cancelStageTransform(): string;
  hostKeyEvent(
    keyCode: number,
    modifierBits: number,
    chars: string,
    isRepeat: boolean,
    timelineFocused: boolean,
  ): number;
}

export default TurboModuleRegistry.get<Spec>('NativeMotoliiHost');
