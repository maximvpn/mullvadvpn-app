import * as React from 'react';
import styled from 'styled-components';

import { Flex, LabelTiny, TitleBig } from '../lib/components';
import { Colors, Spacings } from '../lib/foundations';

export const HeaderTitle = styled(TitleBig)({
  wordWrap: 'break-word',
  hyphens: 'auto',
});

export const HeaderSubTitle = styled(LabelTiny).attrs({
  color: Colors.white60,
})({});

interface SettingsHeaderProps {
  children?: React.ReactNode;
  className?: string;
}

function SettingsHeader(props: SettingsHeaderProps, forwardRef: React.Ref<HTMLDivElement>) {
  return (
    <Flex
      ref={forwardRef}
      $flexDirection="column"
      $gap={Spacings.small}
      $margin={{
        horizontal: Spacings.medium,
        bottom: Spacings.small,
      }}
      className={props.className}>
      {props.children}
    </Flex>
  );
}

export default React.forwardRef(SettingsHeader);
