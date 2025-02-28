import * as React from 'react';
import styled from 'styled-components';

import { Icon } from '../lib/components';
import { DeprecatedColors } from '../lib/foundations';

interface IProps extends React.HTMLAttributes<HTMLButtonElement> {
  up: boolean;
}

const Button = styled.button({
  border: 'none',
  background: 'none',
});

const StyledIcon = styled(Icon)({
  flex: 0,
  alignSelf: 'stretch',
  justifyContent: 'center',
  '&&:hover': {
    backgroundColor: DeprecatedColors.white,
  },
});

export default function ChevronButton(props: IProps) {
  const { up, ...otherProps } = props;

  return (
    <Button {...otherProps}>
      <StyledIcon color={DeprecatedColors.white60} icon={up ? 'chevron-up' : 'chevron-down'} />
    </Button>
  );
}
