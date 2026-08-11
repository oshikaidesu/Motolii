import {codegenNativeComponent} from 'react-native';
import type {HostComponent, ViewProps} from 'react-native';

export interface NativeProps extends ViewProps {}

export default codegenNativeComponent<NativeProps>(
  'MotoliiGpuView',
) as HostComponent<NativeProps>;
