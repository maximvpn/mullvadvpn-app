import styled from 'styled-components';

import { DeprecatedColors } from '../lib/foundations';
import { measurements, normalText, tinyText } from './common-styles';

export const AccountContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
});

export const AccountRows = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
});

export const AccountRow = styled.div({
  padding: `0 ${measurements.horizontalViewMargin}`,
  marginBottom: measurements.rowVerticalMargin,
});

const AccountRowText = styled.span({
  display: 'block',
  fontFamily: 'Open Sans',
});

export const AccountRowLabel = styled(AccountRowText)(tinyText, {
  lineHeight: '20px',
  marginBottom: '5px',
  color: DeprecatedColors.white60,
});

export const AccountRowValue = styled(AccountRowText)(normalText, {
  fontWeight: 600,
  color: DeprecatedColors.white,
});

export const DeviceRowValue = styled(AccountRowValue)({
  textTransform: 'capitalize',
});

export const AccountOutOfTime = styled(AccountRowValue)({
  color: DeprecatedColors.red,
});

export const StyledDeviceNameRow = styled.div({
  display: 'flex',
  flexDirection: 'row',
});
