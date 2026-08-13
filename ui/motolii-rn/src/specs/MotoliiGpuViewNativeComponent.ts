import {codegenNativeComponent} from 'react-native';
import type {CodegenTypes, HostComponent, ViewProps} from 'react-native';

export type StageDropEvent = Readonly<{
  x: CodegenTypes.Double;
  y: CodegenTypes.Double;
  canonicalX: CodegenTypes.Double;
  canonicalY: CodegenTypes.Double;
}>;

export type StageTransformEvent = Readonly<{
  x: CodegenTypes.Double;
  y: CodegenTypes.Double;
  z: CodegenTypes.Double;
  rotationX: CodegenTypes.Double;
  rotationY: CodegenTypes.Double;
  rotationZ: CodegenTypes.Double;
}>;

export type HostTerminalEvent = Readonly<{
  accepted: boolean;
  message: string;
}>;

export interface NativeProps extends ViewProps {
  createdItemId: string;
  draggedItemId: string;
  transformX: CodegenTypes.Double;
  transformY: CodegenTypes.Double;
  transformZ: CodegenTypes.Double;
  rotationX: CodegenTypes.Double;
  rotationY: CodegenTypes.Double;
  rotationZ: CodegenTypes.Double;
  onStageDrop?: CodegenTypes.DirectEventHandler<StageDropEvent>;
  onStageTransform?: CodegenTypes.DirectEventHandler<StageTransformEvent>;
  onHostTerminal?: CodegenTypes.DirectEventHandler<HostTerminalEvent>;
}

export default codegenNativeComponent<NativeProps>(
  'MotoliiGpuView',
) as HostComponent<NativeProps>;
