import {
  codegenNativeComponent,
  type HostComponent,
  type ViewProps,
} from 'react-native';

export interface NativeProps extends ViewProps {
  hostHandle: string;
  scopeRef: string;
  itemId: string;
}

export default codegenNativeComponent<NativeProps>(
  'MotoliiBrowserDragSourceView',
) as HostComponent<NativeProps>;
