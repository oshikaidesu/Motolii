import {
  codegenNativeComponent,
  type CodegenTypes,
  type HostComponent,
  type ViewProps,
} from 'react-native';

export type TimelineFeedbackEvent = Readonly<{
  objectIndex: CodegenTypes.Int32;
  time: CodegenTypes.Double;
}>;

export type HostTerminalEvent = Readonly<{
  accepted: boolean;
  message: string;
}>;

export interface NativeProps extends ViewProps {
  selectedObjectIndex: CodegenTypes.Int32;
  playhead: CodegenTypes.Double;
  onTimelineFeedback?: CodegenTypes.DirectEventHandler<TimelineFeedbackEvent>;
  onHostTerminal?: CodegenTypes.DirectEventHandler<HostTerminalEvent>;
}

export default codegenNativeComponent<NativeProps>(
  'MotoliiTimelineView',
) as HostComponent<NativeProps>;
