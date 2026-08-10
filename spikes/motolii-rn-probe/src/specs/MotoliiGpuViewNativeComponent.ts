import {codegenNativeComponent} from 'react-native';
import type {HostComponent, ViewProps} from 'react-native';
import type {WithDefault} from 'react-native/Libraries/Types/CodegenTypes';

export interface NativeProps extends ViewProps {
  showPathRectangle?: WithDefault<boolean, false>;
}

export default codegenNativeComponent<NativeProps>(
  'MotoliiGpuView',
) as HostComponent<NativeProps>;
