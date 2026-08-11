import {
  codegenNativeComponent,
  type HostComponent,
  type ViewProps,
} from 'react-native';

export interface NativeProps extends ViewProps {
  hostHandle: string;
}

export default codegenNativeComponent<NativeProps>(
  'MotoliiStageView',
) as HostComponent<NativeProps>;
