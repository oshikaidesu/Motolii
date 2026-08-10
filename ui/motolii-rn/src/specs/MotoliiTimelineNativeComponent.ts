import {
  codegenNativeComponent,
  type HostComponent,
  type ViewProps,
} from 'react-native';

export interface NativeProps extends ViewProps {
  hostHandle: string;
  refreshToken: string;
}

export default codegenNativeComponent<NativeProps>(
  'MotoliiTimelineView',
) as HostComponent<NativeProps>;
