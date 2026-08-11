import {codegenNativeComponent} from 'react-native';
import type {CodegenTypes, HostComponent, ViewProps} from 'react-native';

export type StageDropEvent = Readonly<{
  x: CodegenTypes.Double;
  y: CodegenTypes.Double;
}>;

export interface NativeProps extends ViewProps {
  createdItemId: string;
  draggedItemId: string;
  onStageDrop?: CodegenTypes.DirectEventHandler<StageDropEvent>;
}

export default codegenNativeComponent<NativeProps>(
  'MotoliiGpuView',
) as HostComponent<NativeProps>;
